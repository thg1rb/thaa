import type { RuntimeEntry } from "../model/types";
import { bindingLabel } from "../utils/processDisplay";

export function BindingBadge({
  binding,
}: {
  binding: RuntimeEntry["binding"];
}) {
  return (
    <div className="binding-group">
      <span className={`binding-mark binding-${binding}`} aria-hidden="true" />
      <span className={`binding-badge binding-${binding}`}>
        {bindingLabel(binding)}
      </span>
      <span className="protocol-label">LOCAL TCP</span>
    </div>
  );
}
