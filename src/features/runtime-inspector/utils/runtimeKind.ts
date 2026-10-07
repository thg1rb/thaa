import type { RuntimeKind } from "../model/types";

export function runtimeLabel(runtime: RuntimeKind): string {
  switch (runtime) {
    case "nodeJs":
      return "Node.js";
    case "python":
      return "Python";
    case "java":
      return "Java / JVM";
    case "ruby":
      return "Ruby";
    case "php":
      return "PHP";
  }
}
