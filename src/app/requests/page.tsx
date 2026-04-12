"use client";

import { useState, useEffect, useCallback } from "react";
import { AppLayout } from "../../components/layout/app-layout";
import { Button } from "../../components/ui/button";
import { Card } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Input } from "../../components/ui/input";
import {
  MAX_RULE_NAME_LENGTH,
  MAX_MATCH_PATTERN_LENGTH,
  MAX_ACTION_TARGET_LENGTH,
  MAX_ACTION_VALUE_LENGTH,
  validateRuleName,
  validateMatchPattern,
  validateActionTarget,
  validateActionValue,
} from "../../components/ui/input";
import {
  getRules,
  addRule,
  removeRule,
  toggleRule,
  type InterceptRule,
  type RuleAction,
} from "../../lib/tauri-api";
import type { NavItem } from "../../types";

const navItems: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "◉" },
  { id: "proxy", label: "Proxy", icon: "⇄" },
  { id: "interceptor", label: "Interceptor", icon: "◎" },
  { id: "requests", label: "Requests", icon: "▤" },
  { id: "settings", label: "Settings", icon: "⚙" },
];

export default function RulesPage() {
  const [activeNav, setActiveNav] = useState("requests");
  const [rules, setRules] = useState<InterceptRule[]>([]);
  const [showForm, setShowForm] = useState(false);

  // New rule form state
  const [ruleName, setRuleName] = useState("");
  const [matchType, setMatchType] = useState("url_contains");
  const [matchPattern, setMatchPattern] = useState("");
  const [actions, setActions] = useState<RuleAction[]>([
    { action_type: "add_header", target: "", value: "" },
  ]);
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});

  const refreshRules = useCallback(async () => {
    try {
      const r = await getRules();
      setRules(r);
    } catch {
      // Dev mode
    }
  }, []);

  useEffect(() => {
    refreshRules();
  }, [refreshRules]);

  const handleAddRule = async () => {
    // Validate all fields
    const errors: Record<string, string> = {};
    
    const nameError = validateRuleName(ruleName);
    if (nameError) errors.ruleName = nameError;
    
    const patternError = validateMatchPattern(matchPattern);
    if (patternError) errors.matchPattern = patternError;
    
    for (let i = 0; i < actions.length; i++) {
      const targetError = validateActionTarget(actions[i].target);
      if (targetError) errors[`action_${i}_target`] = targetError;
      
      const valueError = validateActionValue(actions[i].value);
      if (valueError) errors[`action_${i}_value`] = valueError;
    }

    if (Object.keys(errors).length > 0) {
      setFormErrors(errors);
      return;
    }

    setFormErrors({});
    try {
      await addRule(ruleName, matchType, matchPattern, actions);
      setRuleName("");
      setMatchPattern("");
      setActions([{ action_type: "add_header", target: "", value: "" }]);
      setShowForm(false);
      refreshRules();
    } catch {
      // Dev mode
    }
  };

  const validateForm = () => {
    const errors: Record<string, string> = {};
    
    const nameError = validateRuleName(ruleName);
    if (nameError) errors.ruleName = nameError;
    
    const patternError = validateMatchPattern(matchPattern);
    if (patternError) errors.matchPattern = patternError;

    setFormErrors(errors);
  };

  const handleRemove = async (id: string) => {
    try {
      await removeRule(id);
      refreshRules();
    } catch {
      // Dev mode
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await toggleRule(id, !enabled);
      refreshRules();
    } catch {
      // Dev mode
    }
  };

  const updateAction = (index: number, field: keyof RuleAction, value: string) => {
    const updated = [...actions];
    updated[index] = { ...updated[index], [field]: value };
    setActions(updated);
  };

  const addAction = () => {
    setActions([...actions, { action_type: "add_header", target: "", value: "" }]);
  };

  const removeAction = (index: number) => {
    setActions(actions.filter((_, i) => i !== index));
  };

  return (
    <AppLayout
      navItems={navItems}
      activeItem={activeNav}
    >
      <div className="flex flex-col h-screen">
        {/* Toolbar */}
        <div className="flex items-center gap-3 px-4 py-2 border-b border-border-primary bg-surface-400">
          <h1
            className="text-xl font-gothic text-cursor-dark"
            style={{ letterSpacing: "-0.3px" }}
          >
            Intercept Rules
          </h1>
          <div className="flex-1" />
          <Button variant="primary" onClick={() => setShowForm(!showForm)}>
            {showForm ? "Cancel" : "Add Rule"}
          </Button>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {/* New rule form */}
          {showForm && (
            <Card className="p-5 mb-6">
              <h2
                className="text-lg font-gothic text-cursor-dark mb-4"
                style={{ letterSpacing: "-0.2px" }}
              >
                New Rule
              </h2>
              <div className="space-y-4">
                <Input
                  label="Rule Name"
                  value={ruleName}
                  onChange={(e) => { setRuleName(e.target.value); setFormErrors(prev => ({ ...prev, ruleName: '' })); }}
                  error={formErrors.ruleName}
                  placeholder="e.g., Add Auth Header"
                  maxLength={MAX_RULE_NAME_LENGTH}
                />
                <div className="grid grid-cols-3 gap-3">
                  <div>
                    <label className="text-xs text-cursor-dark/55 block mb-1">
                      Match Type
                    </label>
                    <select
                      value={matchType}
                      onChange={(e) => setMatchType(e.target.value)}
                      className="w-full bg-transparent border border-border-primary rounded-comfortable px-2 py-1.5 text-sm text-cursor-dark focus:border-border-medium focus:outline-none"
                    >
                      <option value="url_contains">URL Contains</option>
                      <option value="method_equals">Method Equals</option>
                      <option value="url_regex">URL Regex</option>
                      <option value="header_contains">Header Contains</option>
                    </select>
                  </div>
                  <div className="col-span-2">
                    <Input
                      label="Match Pattern"
                      value={matchPattern}
                      onChange={(e) => { setMatchPattern(e.target.value); setFormErrors(prev => ({ ...prev, matchPattern: '' })); }}
                      error={formErrors.matchPattern}
                      placeholder={
                        matchType === "method_equals"
                          ? "GET, POST, etc."
                          : "Pattern..."
                      }
                      maxLength={MAX_MATCH_PATTERN_LENGTH}
                    />
                  </div>
                </div>

                {/* Actions */}
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label className="text-xs text-cursor-dark/55">
                      Actions
                    </label>
                    <button
                      onClick={addAction}
                      className="text-xs text-accent hover:text-error transition-colors"
                    >
                      + Add Action
                    </button>
                  </div>
                  <div className="space-y-2">
                    {actions.map((action, index) => (
                      <div key={index} className="flex gap-2 items-start">
                        <select
                          value={action.action_type}
                          onChange={(e) =>
                            updateAction(index, "action_type", e.target.value)
                          }
                          className="bg-transparent border border-border-primary rounded-comfortable px-2 py-1.5 text-xs text-cursor-dark"
                        >
                          <option value="add_header">Add Header</option>
                          <option value="replace_header">Replace Header</option>
                          <option value="remove_header">Remove Header</option>
                        </select>
                        <input
                          value={action.target}
                          onChange={(e) => {
                            updateAction(index, "target", e.target.value);
                            setFormErrors(prev => { const n = { ...prev }; delete n[`action_${index}_target`]; return n; });
                          }}
                          placeholder="Header name"
                          maxLength={MAX_ACTION_TARGET_LENGTH}
                          className="flex-1 bg-transparent border border-border-primary rounded-comfortable px-2 py-1.5 text-xs text-cursor-dark focus:outline-none"
                        />
                        <input
                          value={action.value}
                          onChange={(e) => {
                            updateAction(index, "value", e.target.value);
                            setFormErrors(prev => { const n = { ...prev }; delete n[`action_${index}_value`]; return n; });
                          }}
                          placeholder="Value"
                          maxLength={MAX_ACTION_VALUE_LENGTH}
                          className="flex-1 bg-transparent border border-border-primary rounded-comfortable px-2 py-1.5 text-xs text-cursor-dark focus:outline-none"
                        />
                        {actions.length > 1 && (
                          <button
                            onClick={() => removeAction(index)}
                            className="text-cursor-dark/30 hover:text-error transition-colors px-2"
                          >
                            ✕
                          </button>
                        )}
                      </div>
                    ))}
                  </div>
                </div>

                <Button variant="primary" onClick={handleAddRule}>
                  Create Rule
                </Button>
              </div>
            </Card>
          )}

          {/* Rules list */}
          <div className="space-y-3">
            {rules.length === 0 ? (
              <Card className="p-8 text-center">
                <p className="text-cursor-dark/40 text-sm">
                  No intercept rules yet. Click "Add Rule" to create one.
                </p>
              </Card>
            ) : (
              rules.map((rule) => (
                <Card key={rule.id} className="p-4">
                  <div className="flex items-center gap-4">
                    <button
                      onClick={() => handleToggle(rule.id, rule.enabled)}
                      className={`w-10 h-5 rounded-pill transition-colors duration-150 ${
                        rule.enabled ? "bg-accent" : "bg-surface-500"
                      }`}
                    >
                      <div
                        className={`w-4 h-4 rounded-full bg-cream shadow-sm transition-transform duration-150 ${
                          rule.enabled ? "translate-x-5" : "translate-x-0.5"
                        }`}
                      />
                    </button>
                    <div className="flex-1">
                      <h3 className="text-sm font-gothic text-cursor-dark">
                        {rule.name}
                      </h3>
                      <p className="text-xs text-cursor-dark/40 font-mono mt-0.5">
                        {rule.match_type}: {rule.match_pattern}
                      </p>
                      <div className="flex gap-1 mt-1">
                        {rule.actions.map((a, i) => (
                          <Badge key={i} color="edit" className="text-[10px]">
                            {a.action_type} {a.target}
                          </Badge>
                        ))}
                      </div>
                    </div>
                    <button
                      onClick={() => handleRemove(rule.id)}
                      className="text-cursor-dark/30 hover:text-error transition-colors p-2"
                    >
                      ✕
                    </button>
                  </div>
                </Card>
              ))
            )}
          </div>
        </div>
      </div>
    </AppLayout>
  );
}
