import type { RuntimeEntry } from "../model/types";
import { bindingDescription, bindingLabel } from "../utils/processDisplay";

export function BindingBadge({
  binding,
  localAddress,
}: {
  binding: RuntimeEntry["binding"];
  localAddress: RuntimeEntry["localAddress"];
}) {
  const label = bindingLabel(binding);
  const description = bindingDescription(binding, localAddress);

  return (
    <div className="binding-group">
      <span className={`binding-mark binding-${binding}`} aria-hidden="true" />
      <span
        className={`binding-badge binding-${binding}`}
        title={description}
        aria-description={description}
      >
        {label}
      </span>
      <span className="protocol-label">LOCAL TCP</span>
    </div>
  );
}
