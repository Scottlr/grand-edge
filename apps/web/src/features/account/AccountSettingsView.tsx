import { FormEvent, useEffect, useState } from "react";

import {
  useCurrentUser,
  useLogin,
  useLogout,
  useRegister,
  useRiskProfile,
  useUpdateRiskProfile,
} from "../../api/hooks";
import type { ExecutionMode } from "../../api/types";

const EXECUTION_MODES: Array<{ value: ExecutionMode; label: string }> = [
  { value: "conservative_instant", label: "Safe default" },
  { value: "passive_estimated", label: "Passive estimate" },
  { value: "haircut_passive", label: "Haircut passive" },
  { value: "worst_case", label: "Worst case" },
  { value: "user_position_replay", label: "Position replay" },
];

export function AccountSettingsView() {
  const currentUser = useCurrentUser();
  const riskProfile = useRiskProfile();
  const login = useLogin();
  const register = useRegister();
  const logout = useLogout();
  const updateRiskProfile = useUpdateRiskProfile();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [mode, setMode] = useState<"login" | "register">("login");
  const [riskForm, setRiskForm] = useState({
    maxGpPerItem: 5_000_000,
    maxPortfolioDrawdown: 0.15,
    minExpectedRoi: 0.01,
    minConfidence: 0.55,
    participationRate: 0.1,
    preferredExecutionMode: "conservative_instant" as ExecutionMode,
  });

  useEffect(() => {
    if (!riskProfile.data) {
      return;
    }
    setRiskForm({
      maxGpPerItem: riskProfile.data.maxGpPerItem,
      maxPortfolioDrawdown: riskProfile.data.maxPortfolioDrawdown,
      minExpectedRoi: riskProfile.data.minExpectedRoi,
      minConfidence: riskProfile.data.minConfidence,
      participationRate: riskProfile.data.participationRate,
      preferredExecutionMode: riskProfile.data.preferredExecutionMode,
    });
  }, [riskProfile.data]);

  function submitAuth(event: FormEvent) {
    event.preventDefault();
    if (mode === "login") {
      login.mutate({ email, password });
      return;
    }
    register.mutate({ email, password, displayName: displayName || undefined });
  }

  function submitRisk(event: FormEvent) {
    event.preventDefault();
    updateRiskProfile.mutate(riskForm);
  }

  return (
    <section className="terminal-panel">
      <p className="eyebrow">Account</p>
      <h2>{currentUser.data ? "Signed in" : "Sign in or create an account"}</h2>
      <p>
        {currentUser.data
          ? `Using ${currentUser.data.email} for personalized holdings and risk settings.`
          : "Sign in to save positions and personal risk limits across sessions."}
      </p>

      {!currentUser.data ? (
        <form className="strategy-lab-card" onSubmit={submitAuth}>
          <label>
            Mode
            <select value={mode} onChange={(event) => setMode(event.target.value as "login" | "register")}>
              <option value="login">Sign in</option>
              <option value="register">Create account</option>
            </select>
          </label>
          <label>
            Email
            <input value={email} onChange={(event) => setEmail(event.target.value)} type="email" />
          </label>
          <label>
            Password
            <input
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              type="password"
            />
          </label>
          {mode === "register" ? (
            <label>
              Display name
              <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} />
            </label>
          ) : null}
          <button type="submit">{mode === "login" ? "Sign in" : "Create account"}</button>
        </form>
      ) : (
        <div className="strategy-lab-card">
          <p>
            Signed in as <strong>{currentUser.data.displayName ?? currentUser.data.email}</strong>
          </p>
          <button type="button" onClick={() => logout.mutate()}>
            Sign out
          </button>
        </div>
      )}

      <form className="strategy-lab-card" onSubmit={submitRisk}>
        <p className="eyebrow">Risk profile</p>
        <label>
          Max GP per item
          <input
            type="number"
            value={riskForm.maxGpPerItem}
            onChange={(event) =>
              setRiskForm((current) => ({ ...current, maxGpPerItem: Number(event.target.value) }))
            }
          />
        </label>
        <label>
          Max drawdown
          <input
            type="number"
            step="0.01"
            value={riskForm.maxPortfolioDrawdown}
            onChange={(event) =>
              setRiskForm((current) => ({
                ...current,
                maxPortfolioDrawdown: Number(event.target.value),
              }))
            }
          />
        </label>
        <label>
          Minimum expected ROI
          <input
            type="number"
            step="0.01"
            value={riskForm.minExpectedRoi}
            onChange={(event) =>
              setRiskForm((current) => ({ ...current, minExpectedRoi: Number(event.target.value) }))
            }
          />
        </label>
        <label>
          Minimum confidence
          <input
            type="number"
            step="0.01"
            value={riskForm.minConfidence}
            onChange={(event) =>
              setRiskForm((current) => ({ ...current, minConfidence: Number(event.target.value) }))
            }
          />
        </label>
        <label>
          Participation rate
          <input
            type="number"
            step="0.01"
            value={riskForm.participationRate}
            onChange={(event) =>
              setRiskForm((current) => ({ ...current, participationRate: Number(event.target.value) }))
            }
          />
        </label>
        <label>
          Preferred execution mode
          <select
            value={riskForm.preferredExecutionMode}
            onChange={(event) =>
              setRiskForm((current) => ({
                ...current,
                preferredExecutionMode: event.target.value as ExecutionMode,
              }))
            }
          >
            {EXECUTION_MODES.map((entry) => (
              <option key={entry.value} value={entry.value}>
                {entry.label}
              </option>
            ))}
          </select>
        </label>
        <button type="submit">Save risk profile</button>
      </form>
    </section>
  );
}
