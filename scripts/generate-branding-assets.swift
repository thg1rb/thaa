#!/usr/bin/env swift
import AppKit
import CoreGraphics
import Foundation
import ImageIO

private let macOSAppIconCanvas = 1024
private let macOSAppIconArtwork = 824
private let macOSTrayCanvas = 44
private let macOSTrayGlyphHeight = 32
private let windowsTrayCanvas = 32
private let windowsTrayGlyphHeight = 23

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
          bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            | CGBitmapInfo.byteOrder32Big.rawValue
        )
      else { return }

      context.clear(CGRect(x: 0, y: 0, width: width, height: height))
      context.interpolationQuality = .high
      // CGImage pixel rows and this bitmap context's data are both top-to-bottom.
      // Do not vertically flip here; that inverted the application master.
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

  func rgba(x: Int, y: Int) -> (UInt8, UInt8, UInt8, UInt8) {
    let offset = (y * width + x) * 4
    return (pixels[offset], pixels[offset + 1], pixels[offset + 2], pixels[offset + 3])
  }

  func visiblePixelsAreMonochrome() -> Bool {
    for y in 0..<height {
      for x in 0..<width {
        let color = rgba(x: x, y: y)
        if color.3 > 0
          && (abs(Int(color.0) - Int(color.1)) > 1 || abs(Int(color.1) - Int(color.2)) > 1)
        {
          return false
        }
      }
    }
    return true
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
      bitmapInfo: CGBitmapInfo(
        rawValue: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue
      ),
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
  guard
    let destination = CGImageDestinationCreateWithURL(
      url as CFURL, "public.png" as CFString, 1, nil)
  else {
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

func imageAlphaBounds(_ image: CGImage) -> AlphaBounds? {
  rasterize(image).alphaBounds()
}

func rasterize(_ image: CGImage) -> PixelCanvas {
  var canvas = PixelCanvas(width: image.width, height: image.height)
  canvas.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
  return canvas
}

func assertFixtureOrientation(_ canvas: PixelCanvas, stage: String) {
  let topLeft = canvas.rgba(x: 0, y: 0)
  let topRight = canvas.rgba(x: 1, y: 0)
  let bottomLeft = canvas.rgba(x: 0, y: 1)
  let bottomRight = canvas.rgba(x: 1, y: 1)
  guard
    topLeft.0 > 240, topLeft.1 < 10, topLeft.2 < 10,
    topRight.0 < 10, topRight.1 > 240, topRight.2 < 10,
    bottomLeft.0 < 10, bottomLeft.1 < 10, bottomLeft.2 > 240,
    bottomRight.0 > 240, bottomRight.1 > 240, bottomRight.2 < 10
  else {
    fail("CoreGraphics/ImageIO changed the fixture's top/bottom orientation at \(stage)")
  }
}

func verifyOrientation() {
  // The provider's first row is the PNG top row: red/green above blue/yellow.
  let fixtureBytes: [UInt8] = [
    255, 0, 0, 255, 0, 255, 0, 255,
    0, 0, 255, 255, 255, 255, 0, 255,
  ]
  let fixtureData = Data(fixtureBytes) as CFData
  guard
    let provider = CGDataProvider(data: fixtureData),
    let fixture = CGImage(
      width: 2,
      height: 2,
      bitsPerComponent: 8,
      bitsPerPixel: 32,
      bytesPerRow: 8,
      space: CGColorSpace(name: CGColorSpace.sRGB)!,
      bitmapInfo: CGBitmapInfo(
        rawValue: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue
      ),
      provider: provider,
      decode: nil,
      shouldInterpolate: false,
      intent: .defaultIntent
    )
  else {
    fail("could not create the orientation fixture")
  }

  var canvas = PixelCanvas(width: 2, height: 2)
  canvas.draw(fixture, in: CGRect(x: 0, y: 0, width: 2, height: 2))
  assertFixtureOrientation(canvas, stage: "bitmap drawing")

  guard let rendered = canvas.cgImage() else { fail("could not render the orientation fixture") }
  let encodedData = NSMutableData()
  guard
    let destination = CGImageDestinationCreateWithData(
      encodedData, "public.png" as CFString, 1, nil)
  else {
    fail("could not create an in-memory PNG orientation fixture")
  }
  CGImageDestinationAddImage(destination, rendered, nil)
  guard
    CGImageDestinationFinalize(destination),
    let source = CGImageSourceCreateWithData(encodedData, nil),
    let decoded = CGImageSourceCreateImageAtIndex(source, 0, nil)
  else {
    fail("could not round-trip the PNG orientation fixture")
  }
  assertFixtureOrientation(rasterize(decoded), stage: "PNG encode/decode")
  print("orientation fixture: passed through drawing and PNG encode/decode")
}

func cropToAlphaBounds(_ image: CGImage) -> (CGImage, AlphaBounds) {
  var canvas = PixelCanvas(width: image.width, height: image.height)
  canvas.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
  guard let bounds = canvas.alphaBounds(), let normalized = canvas.cgImage() else {
    fail("source image has no visible alpha content")
  }
  let rect = CGRect(x: bounds.minX, y: bounds.minY, width: bounds.width, height: bounds.height)
  guard let cropped = normalized.cropping(to: rect) else {
    fail("could not crop visible tray artwork")
  }
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
  var args = Array(CommandLine.arguments.dropFirst())

  while !args.isEmpty {
    let argument = args.removeFirst()
    switch argument {
    case "--output-dir":
      guard let path = args.first else { fail("--output-dir requires a path") }
      args.removeFirst()
      outputDirectory =
        URL(
          fileURLWithPath: path,
          relativeTo: URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        )
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
  guard appSource.width == macOSAppIconCanvas, appSource.height == macOSAppIconCanvas else {
    fail("application master must remain 1024×1024")
  }

  verifyOrientation()

  let appOrigin = (macOSAppIconCanvas - macOSAppIconArtwork) / 2
  let appRect = CGRect(
    x: appOrigin, y: appOrigin, width: macOSAppIconArtwork, height: macOSAppIconArtwork)
  let appOutput = render(
    appSource, width: macOSAppIconCanvas, height: macOSAppIconCanvas, destination: appRect)
  guard
    let appBounds = imageAlphaBounds(appOutput),
    appBounds.minX == appOrigin, appBounds.minY == appOrigin,
    appBounds.width == macOSAppIconArtwork, appBounds.height == macOSAppIconArtwork
  else {
    fail("application derivative does not have the expected centered 824×824 alpha bounds")
  }
  let appPath = outputDirectory.appendingPathComponent("thaa-app-icon-production.png")
  savePNG(appOutput, to: appPath)
  print(
    "\(appPath.path): canvas=\(macOSAppIconCanvas)×\(macOSAppIconCanvas), artwork=\(macOSAppIconArtwork)×\(macOSAppIconArtwork), alphaBounds=(\(appBounds.minX),\(appBounds.minY))..(\(appBounds.maxX),\(appBounds.maxY)), margin=100px"
  )

  for (sourceName, outputName, outputSize, glyphHeight, minimumSideMargin) in [
    (
      "thaa-tray-template-macos.png", "thaa-tray-template-macos.png", macOSTrayCanvas,
      macOSTrayGlyphHeight, 1
    ),
    (
      "thaa-tray-windows.png", "thaa-tray-windows.png", windowsTrayCanvas, windowsTrayGlyphHeight, 1
    ),
  ] {
    let source = loadPNG(iconRoot.appendingPathComponent("source/\(sourceName)"))
    let (cropped, bounds) = cropToAlphaBounds(source)
    let width = CGFloat(cropped.width) * CGFloat(glyphHeight) / CGFloat(cropped.height)
    guard width <= CGFloat(outputSize - 2 * minimumSideMargin) else {
      fail(
        "cropped artwork does not fit the \(outputSize)px tray canvas with the required side margin"
      )
    }
    let x = (CGFloat(outputSize) - width) / 2
    let y = (CGFloat(outputSize - glyphHeight)) / 2
    let result = render(
      cropped, width: outputSize, height: outputSize,
      destination: CGRect(x: x, y: y, width: width, height: CGFloat(glyphHeight)))
    let outputCanvas = rasterize(result)
    guard
      let outputBounds = outputCanvas.alphaBounds(),
      outputBounds.width <= outputSize - 2 * minimumSideMargin,
      outputBounds.height >= glyphHeight - 1, outputBounds.height <= glyphHeight + 1
    else {
      fail("\(sourceName) output alpha bounds exceed its size or safe margin")
    }
    if sourceName == "thaa-tray-template-macos.png" && !outputCanvas.visiblePixelsAreMonochrome() {
      fail("macOS template tray output must remain monochrome")
    }
    let path = outputDirectory.appendingPathComponent(outputName)
    savePNG(result, to: path)
    print(
      "\(sourceName): source=\(source.width)×\(source.height), alphaBounds=(\(bounds.minX),\(bounds.minY))..(\(bounds.maxX),\(bounds.maxY)), crop=\(bounds.width)×\(bounds.height)"
    )
    print(
      "\(path.path): canvas=\(outputSize)×\(outputSize), alphaBounds=(\(outputBounds.minX),\(outputBounds.minY))..(\(outputBounds.maxX),\(outputBounds.maxY)), targetGlyphHeight=\(glyphHeight)px, preservedAspectRatio=true"
    )
  }
}

main()
