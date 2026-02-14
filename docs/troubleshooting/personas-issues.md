# Persona Troubleshooting Guide

This guide covers common issues with the agent persona system and how to diagnose and fix them.

## Issue 1: Persona Not Reflected in Agent Responses

**Symptom:** Agent responds with generic text instead of its defined personality.

### Diagnosis Steps

1. **Check if SOUL.md exists:**
   ```bash
   ls workspace/SOUL.md
   ```
   If missing, agents use default personality (generic traits from AGENTS.md only).

2. **Check daemon logs for loading:**
   ```bash
   RUST_LOG=debug aofctl serve --workspace workspace/ 2>&1 | grep -i "soul\|persona\|prompt"
   ```
   Look for:
   - `"SOUL.md not found"` -- File missing, using defaults
   - `"Prompt cache miss for agent"` -- Prompt was composed (good)
   - `"Persona validation failed"` -- SOUL.md has errors

3. **Verify SOUL.md format:**
   The YAML block must be inside triple-backtick yaml fences:
   ```
   ## agent-id

   ```yaml         <-- must be exactly this
   id: agent-id
   ...
   ```             <-- closing fence required
   ```
   Missing fences cause the section to be skipped silently.

4. **Check ID match:**
   The `id` in the SOUL.md YAML block must exactly match the `id` in AGENTS.md:
   ```bash
   # Compare IDs
   grep "id:" workspace/AGENTS.md
   grep "id:" workspace/SOUL.md
   ```

### Solutions

- **Missing SOUL.md:** Create the file following the format in the [tutorial](../tutorials/create-agent-persona.md)
- **Format error:** Ensure YAML is inside ` ```yaml ... ``` ` fences
- **ID mismatch:** Make the SOUL.md id match AGENTS.md exactly (case-sensitive, hyphen-sensitive)
- **Reload daemon:** After fixing, restart the daemon or save the file to trigger the file watcher

## Issue 2: Metrics Showing as "--" (null)

**Symptom:** AgentCard in Mission Control shows "--" instead of uptime/success percentages.

### Diagnosis Steps

1. **Check event count:**
   ```bash
   curl http://localhost:3030/api/agents/YOUR-AGENT-ID/metrics | jq .event_count
   ```
   Metrics require at least 10 events. Below this threshold, percentages are `null`.

2. **Check if events are being recorded:**
   ```bash
   # Watch WebSocket for activity events
   websocat ws://localhost:3030/ws | jq '.activity.activity_type'
   ```
   You should see `"Completed"`, `"Error"`, etc. as the agent runs tasks.

3. **Check cache version:**
   ```bash
   curl -I http://localhost:3030/api/agents/YOUR-AGENT-ID/metrics
   # Look for X-Metrics-Version header
   ```
   Version should increase with each event.

### Solutions

- **Insufficient events:** Run more agent tasks. After 10 events, metrics will compute.
- **No events recording:** Verify the agent is actually running tasks through the executor.
- **Cache not updating:** Check that the ReliabilityCache subscriber is connected to the broadcast channel (daemon startup logs).

## Issue 3: Avatar Emoji Rendering Wrong

**Symptom:** Avatar shows as a box, question mark, or multiple characters in the UI.

### Diagnosis Steps

1. **Check the emoji value:**
   ```bash
   curl http://localhost:3030/api/config/agents | jq '.agents[] | {id, avatar}'
   ```

2. **Verify it's a single grapheme cluster:**
   The avatar must be exactly one emoji. Multi-character sequences (like flag emoji or some ZWJ sequences) may render as multiple characters on some platforms.

### Solutions

- **Use common emoji:** Stick to these well-supported choices:
  | Emoji | Unicode | Name |
  |-------|---------|------|
  | (robot) | `\U0001F916` | Robot |
  | (magnifying glass) | `\U0001F50D` | Magnifying glass |
  | (alarm) | `\U0001F6A8` | Rotating light |
  | (elephant) | `\U0001F418` | Elephant |
  | (test tube) | `\U0001F9EA` | Test tube |
  | (shield) | `\U0001F6E1` | Shield |
  | (gear) | `\U00002699` | Gear |
  | (wrench) | `\U0001F527` | Wrench |
  | (bear) | `\U0001F43B` | Bear |
  | (snake) | `\U0001F40D` | Snake |

- **Avoid complex emoji:** Flag sequences, skin tone modifiers, and ZWJ sequences may not display consistently.
- **Test in target browser:** Open Mission Control and verify the emoji renders correctly.

## Issue 4: Introduction Message Not Appearing

**Symptom:** No introduction toast in Mission Control when daemon starts.

### Diagnosis Steps

1. **Check SOUL.md default_intro:**
   ```bash
   grep "default_intro" workspace/SOUL.md
   ```
   Empty `default_intro` triggers a fallback message. Missing SOUL.md skips intros.

2. **Check WebSocket connection:**
   ```bash
   websocat ws://localhost:3030/ws
   ```
   Restart the daemon and watch for introduction events. You should see JSON with `"introduction"` field.

3. **Check daemon startup logs:**
   ```bash
   RUST_LOG=info aofctl serve 2>&1 | grep -i "introduction\|intro\|emit"
   ```
   Look for `"Emitting introduction events"` or `"Agent introduction"` lines.

4. **Check UI WebSocket state:**
   Open browser DevTools (F12) > Network tab > WS. Check if the WebSocket connection is established and receiving messages.

### Solutions

- **SOUL.md missing:** Create the file with `default_intro` for each agent.
- **WebSocket not connected:** Check that Mission Control is running and connected to the correct daemon URL.
- **Toasts dismissed too fast:** Introduction toasts auto-dismiss after 8 seconds. Maximum 3 toasts display simultaneously; extras are queued.
- **Events already sent:** Introduction events fire only at daemon startup. If you connected the UI after startup, you missed them. Restart the daemon.

## Issue 5: Skill Not Found in TOOLS.md

**Symptom:** Daemon logs warn: `"Skill 'X' not found in TOOLS.md for agent 'Y'"`.

### Diagnosis Steps

1. **Check agent skills:**
   ```bash
   curl http://localhost:3030/api/config/agents | jq '.agents[] | select(.id == "YOUR-AGENT") | .skills'
   ```

2. **Check TOOLS.md entries:**
   ```bash
   grep "name:" workspace/TOOLS.md
   ```

3. **Compare names:**
   Skill names must be lowercase-hyphenated and match tool names exactly.

### Solutions

- **Add the tool:** Add the missing tool to `workspace/TOOLS.md`:
  ```yaml
  - name: missing-tool-name
    description: What this tool does
    category: its-category
  ```

- **Fix the skill name:** Update the agent's `skills` list in AGENTS.md to match the tool name in TOOLS.md.

- **Not critical:** This is a warning, not an error. The prompt will still compose. The tool will appear as "not found in TOOLS.md" in the agent's system prompt.

## Issue 6: Prompt Too Long, Truncation Occurred

**Symptom:** Daemon logs warn: `"Persona prompt truncation needed for agent 'X'"`.

### Diagnosis Steps

1. **Check prompt size:**
   ```bash
   RUST_LOG=debug aofctl serve 2>&1 | grep "truncat"
   ```
   The log shows the original and truncated token counts.

2. **Identify the cause:**
   - Many skills (50+) produce long tool sections
   - Long communication guides add prose to the prompt
   - Multiple long boundary statements increase prompt size

### Solutions

- **Reduce skills:** Limit agents to 10-15 skills. If an agent needs more, split into specialized sub-agents.

- **Shorten communication guide:** Keep the prose concise. Focus on the most important communication patterns.

- **Split agents:** Instead of one agent with 50 skills, create 2-3 focused agents with 10-15 skills each.

- **Check truncation priority:** The system removes sections in this order (lowest priority first):
  1. Behavioral rules (generic "always explain reasoning...")
  2. Tool descriptions (shortened to tool names only)
  3. Communication guide (the prose section)
  4. Base, role, personality, boundaries (NEVER removed)

- **Expert override:** Set `system_prompt` directly in the agent config to bypass composition entirely.

## Issue 7: Validation Errors on Startup

**Symptom:** Daemon fails to start with persona validation errors.

### Common Errors and Fixes

| Error Message | Cause | Fix |
|--------------|-------|-----|
| `agents[N].id: must be lowercase-hyphenated` | ID contains uppercase or underscores | Use format `my-agent-name` |
| `agents[N].avatar: 'X' is not a single emoji` | Avatar is text or multiple characters | Use single emoji like `"\U0001F916"` |
| `agents[N].id: duplicate id 'X'` | Two agents share the same ID | Give each agent a unique ID |
| `agents[N].personality_traits: must have at least one trait` | Empty traits list | Add at least one personality trait |
| `soul[X].id: 'X' does not match any agent id` | SOUL.md has an entry with no matching AGENTS.md agent | Add the agent to AGENTS.md or remove the SOUL.md section |
| `potential prompt injection detected` | SOUL.md text contains suspicious patterns | Remove text like "ignore all previous instructions" |

### Quick Fix Checklist

1. Every agent has: `id`, `name`, `role`, `avatar`, `personality_traits` (1+), `can` (1+), `cannot` (1+), `skills` (1+)
2. All IDs are lowercase, start with a letter, use only letters/numbers/hyphens
3. No duplicate IDs across agents
4. Every SOUL.md ID matches an AGENTS.md ID
5. No prompt injection patterns in any text field

## Issue 8: File Watcher Not Detecting Changes

**Symptom:** You edited SOUL.md or AGENTS.md but the daemon didn't reload.

### Diagnosis Steps

1. **Check daemon logs:**
   ```bash
   RUST_LOG=debug aofctl serve 2>&1 | grep -i "watch\|reload\|change"
   ```
   Look for `"Persona file change detected"` or `"Reloading persona files"`.

2. **Check file modification:**
   ```bash
   # Verify the file was actually modified
   stat workspace/SOUL.md
   ```

### Solutions

- **Editor buffering:** Some editors (vim with swap files, VS Code with auto-save delay) may not trigger filesystem events immediately. Try saving explicitly.

- **Restart daemon:** If the watcher is not responding, restart `aofctl serve`. The daemon reloads all files on startup.

- **Manual trigger:** Touch the file to trigger a filesystem event:
  ```bash
  touch workspace/SOUL.md
  ```

- **Debounce delay:** The watcher coalesces rapid changes with a 100ms debounce. If you're editing very quickly, changes may be delayed slightly.

## Still Stuck?

1. **Enable full debug logging:**
   ```bash
   RUST_LOG=debug,aof_personas=trace aofctl serve 2>&1 | tee daemon.log
   ```

2. **Check the log file** for errors related to persona loading, validation, or composition.

3. **Open an issue** on [GitHub](https://github.com/agenticdevops/aof/issues) with:
   - Your AGENTS.md and SOUL.md (redacted if sensitive)
   - The daemon log output
   - What you expected vs what happened
