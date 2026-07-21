export interface DeployTxResult {
  success: boolean;
  status?: string;
  message: string;
  affectedRows?: number;
}

export function buildDeployTxResult(txLog: any, t: (key: string, params?: Record<string, any>) => string): DeployTxResult {
  const status = txLog?.status;
  if (status === "committed") {
    return { success: true, status, message: t("diff.executeSuccess") };
  }
  if (status === "mixed") {
    return {
      success: false,
      status,
      message: t("diff.deployMixed", { participants: txLog?.participants?.length ?? 0 }),
    };
  }
  if (status === "rolled_back") {
    return { success: false, status, message: t("diff.deployRolledBack") };
  }
  return { success: false, status: status || "unknown", message: t("diff.deployFailed", { status: status || "unknown" }) };
}
