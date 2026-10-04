#!/usr/bin/env swift

import AppKit
import CoreGraphics
import Foundation
import ImageIO

struct AlphaBounds {
  let minX: Int
  let minY: Int
  let maxX: Int
  let maxY: Int

  var width: Int { maxX - minX + 1 }
  var height: Int { maxY - minY + 1 }
}

struct PixelCanvas {
  let width: Int
  let height: Int
  private(set) var pixels: [UInt8]

  init(width: Int, height: Int) {
    self.width = width
    self.height = height
    self.pixels = Array(repeating: 0, count: width * height * 4)
  }

  mutating func draw(_ image: CGImage, in destination: CGRect) {
    pixels.withUnsafeMutableBytes { buffer in
      guard
        let baseAddress = buffer.baseAddress,
        let context = CGContext(
          data: baseAddress,
          width: width,
          height: height,
          bitsPerComponent: 8,
          bytesPerRow: width * 4,
          space: CGColorSpace(name: CGColorSpace.sRGB)!,
          bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue
        )
      else { return }

      context.clear(CGRect(x: 0, y: 0, width: width, height: height))
      context.interpolationQuality = .high
      context.translateBy(x: 0, y: CGFloat(height))
      context.scaleBy(x: 1, y: -1)
      context.draw(image, in: destination)
    }
  }

  func alphaBounds() -> AlphaBounds? {
    var minX = width
    var minY = height
    var maxX = -1
    var maxY = -1

    for y in 0..<height {
      for x in 0..<width where pixels[(y * width + x) * 4 + 3] > 0 {
        minX = min(minX, x)
        minY = min(minY, y)
        maxX = max(maxX, x)
        maxY = max(maxY, y)
      }
    }

    guard maxX >= minX, maxY >= minY else { return nil }
    return AlphaBounds(minX: minX, minY: minY, maxX: maxX, maxY: maxY)
  }

  func cgImage() -> CGImage? {
    let data = Data(pixels) as CFData
    guard let provider = CGDataProvider(data: data) else { return nil }

    return CGImage(
        width: width,
        height: height,
        bitsPerComponent: 8,
        bitsPerPixel: 32,
        bytesPerRow: width * 4,
        space: CGColorSpace(name: CGColorSpace.sRGB)!,
        bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue),
        provider: provider,
        decode: nil,
        shouldInterpolate: true,
        intent: .defaultIntent
      )
  }
}

func fail(_ message: String) -> Never {
  fputs("branding asset generation failed: \(message)\n", stderr)
  exit(EXIT_FAILURE)
}

func loadPNG(_ url: URL) -> CGImage {
  guard
    let source = CGImageSourceCreateWithURL(url as CFURL, nil),
    let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
  else { fail("could not decode PNG at \(url.path)") }
  return image
}

func savePNG(_ image: CGImage, to url: URL) {
  guard let destination = CGImageDestinationCreateWithURL(url as CFURL, "public.png" as CFString, 1, nil) else {
    fail("could not create PNG destination at \(url.path)")
  }
  CGImageDestinationAddImage(destination, image, nil)
  guard CGImageDestinationFinalize(destination) else { fail("could not write PNG at \(url.path)") }
}

func render(_ image: CGImage, width: Int, height: Int, destination: CGRect) -> CGImage {
  var canvas = PixelCanvas(width: width, height: height)
  canvas.draw(image, in: destination)
  guard let output = canvas.cgImage() else { fail("could not create \(width)×\(height) image") }
  return output
}

func cropToAlphaBounds(_ image: CGImage) -> (CGImage, AlphaBounds) {
  var canvas = PixelCanvas(width: image.width, height: image.height)
  canvas.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
  guard let bounds = canvas.alphaBounds(), let normalized = canvas.cgImage() else {
    fail("source image has no visible alpha content")
  }
  let rect = CGRect(x: bounds.minX, y: bounds.minY, width: bounds.width, height: bounds.height)
  guard let cropped = normalized.cropping(to: rect) else { fail("could not crop visible tray artwork") }
  return (cropped, bounds)
}

func repositoryRoot() -> URL {
  URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
}

func main() {
  #if !os(macOS)
    fail("this asset-generation script uses the macOS CoreGraphics/ImageIO SDK")
  #endif

  let root = repositoryRoot()
  var outputDirectory = root.appendingPathComponent("src-tauri/icons/derived", isDirectory: true)
  var occupancy: CGFloat = 0.84
  var writeCandidates = false
  var args = Array(CommandLine.arguments.dropFirst())

  while !args.isEmpty {
    let argument = args.removeFirst()
    switch argument {
    case "--app-occupancy":
      guard let rawValue = args.first, let value = Double(rawValue) else {
        fail("--app-occupancy requires 0.82, 0.84, or 0.86")
      }
      args.removeFirst()
      guard [0.82, 0.84, 0.86].contains(value) else {
        fail("supported app occupancies are 0.82, 0.84, and 0.86")
      }
      occupancy = CGFloat(value)
    case "--compare-app-sizes":
      writeCandidates = true
    case "--output-dir":
      guard let path = args.first else { fail("--output-dir requires a path") }
      args.removeFirst()
      outputDirectory = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: FileManager.default.currentDirectoryPath))
        .standardizedFileURL
    default:
      fail("unknown argument \(argument)")
    }
  }

  do {
    try FileManager.default.createDirectory(at: outputDirectory, withIntermediateDirectories: true)
  } catch {
    fail("could not create output directory: \(error.localizedDescription)")
  }

  let iconRoot = root.appendingPathComponent("src-tauri/icons", isDirectory: true)
  let appSource = loadPNG(iconRoot.appendingPathComponent("source/thaa-app-icon.png"))
  guard appSource.width == 1024, appSource.height == 1024 else {
    fail("application master must remain 1024×1024")
  }

  func writeAppIcon(_ scale: CGFloat, name: String) {
    let size = CGFloat(1024) * scale
    let origin = (CGFloat(1024) - size) / 2
    let result = render(appSource, width: 1024, height: 1024, destination: CGRect(x: origin, y: origin, width: size, height: size))
    let path = outputDirectory.appendingPathComponent(name)
    savePNG(result, to: path)
    print("\(path.path): canvas=1024×1024, artwork=\(Int(size.rounded()))×\(Int(size.rounded())), margin=\(Int(origin.rounded()))px, occupancy=\(Int((scale * 100).rounded()))%")
  }

  writeAppIcon(occupancy, name: "thaa-app-icon-production.png")
  if writeCandidates {
    for candidate: CGFloat in [0.82, 0.84, 0.86] {
      writeAppIcon(candidate, name: "thaa-app-icon-\(Int(candidate * 100)).png")
    }
  }

  for (sourceName, outputName, outputSize, glyphHeight) in [
    ("thaa-tray-template-macos.png", "thaa-tray-template-macos.png", 36, 26),
    ("thaa-tray-windows.png", "thaa-tray-windows.png", 32, 23),
  ] {
    let source = loadPNG(iconRoot.appendingPathComponent("source/\(sourceName)"))
    let (cropped, bounds) = cropToAlphaBounds(source)
    let width = CGFloat(cropped.width) * CGFloat(glyphHeight) / CGFloat(cropped.height)
    guard width < CGFloat(outputSize) else { fail("cropped artwork does not fit the \(outputSize)px tray canvas") }
    let x = (CGFloat(outputSize) - width) / 2
    let y = (CGFloat(outputSize - glyphHeight)) / 2
    let result = render(cropped, width: outputSize, height: outputSize, destination: CGRect(x: x, y: y, width: width, height: CGFloat(glyphHeight)))
    let path = outputDirectory.appendingPathComponent(outputName)
    savePNG(result, to: path)
    print("\(sourceName): source=\(source.width)×\(source.height), alphaBounds=(\(bounds.minX),\(bounds.minY))..(\(bounds.maxX),\(bounds.maxY)), crop=\(bounds.width)×\(bounds.height)")
    print("\(path.path): canvas=\(outputSize)×\(outputSize), targetGlyphHeight=\(glyphHeight)px, glyphHeightOccupancy=\(String(format: "%.1f", Double(glyphHeight) / Double(outputSize - 6) * 100))% of usable height")
  }
}

main()
