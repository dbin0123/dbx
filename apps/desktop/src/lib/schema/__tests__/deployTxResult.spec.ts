import { describe, expect, it } from "vitest";
import { buildDeployTxResult } from "@/lib/schema/deployTxResult";

const t = (key: string, _params?: Record<string, any>) => {
  const fallback: Record<string, string> = {
    "diff.executeSuccess": "Executed successfully",
    "diff.deployMixed": "Partially deployed",
    "diff.deployRolledBack": "Rolled back",
    "diff.deployFailed": "Deployment failed: {status}",
  };
  return fallback[key] || key;
};

describe("buildDeployTxResult", () => {
  it("returns success for committed transaction", () => {
    const result = buildDeployTxResult({ status: "committed", transaction_id: "tx1" }, t);
    expect(result.success).toBe(true);
    expect(result.status).toBe("committed");
    expect(result.message).toBe("Executed successfully");
  });

  it("returns failure with mixed status for partially committed", () => {
    const result = buildDeployTxResult({ status: "mixed", participants: [{ id: "1" }, { id: "2" }] }, t);
    expect(result.success).toBe(false);
    expect(result.status).toBe("mixed");
    expect(result.message).toBe("Partially deployed");
  });

  it("returns failure with rolled_back status", () => {
    const result = buildDeployTxResult({ status: "rolled_back" }, t);
    expect(result.success).toBe(false);
    expect(result.status).toBe("rolled_back");
    expect(result.message).toBe("Rolled back");
  });

  it("returns failure for unknown status", () => {
    const result = buildDeployTxResult({ status: "unknown" }, t);
    expect(result.success).toBe(false);
    expect(result.status).toBe("unknown");
    expect(result.message).toContain("unknown");
  });

  it("returns failure for null/undefined txLog", () => {
    const result = buildDeployTxResult(null, t);
    expect(result.success).toBe(false);
    expect(result.status).toBe("unknown");
  });
});
