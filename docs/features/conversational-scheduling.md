# Conversational Scheduling

Set up automated agent execution schedules through natural language conversation.

## What Is Schedule Configuration?

Instead of writing cron expressions manually, describe when you want your agent to run in plain English. The system converts your description into a validated cron schedule with timezone support and shows you the next 3 run times for confirmation.

## Supported Patterns

| Natural Language | Cron Expression | Description |
|-----------------|-----------------|-------------|
| `every 30 minutes` | `0 */30 * * * *` | Run every 30 minutes |
| `every 5 hours` | `0 0 */5 * * *` | Run every 5 hours |
| `daily at 6am` | `0 0 6 * * *` | Run once daily at 6:00 AM |
| `daily at 6pm` | `0 0 18 * * *` | Run once daily at 6:00 PM |
| `daily at noon` | `0 0 12 * * *` | Run at 12:00 PM |
| `daily at midnight` | `0 0 0 * * *` | Run at 12:00 AM |
| `every weekday at 9am` | `0 0 9 * * 1-5` | Run Monday-Friday at 9:00 AM |
| `every Monday and Friday` | `0 0 0 * * 1,5` | Run on Mondays and Fridays |
| `business hours` | `0 0 9-17 * * 1-5` | Run every hour from 9 AM to 5 PM on weekdays |
| `3 times per day` | `0 0 6,12,18 * * *` | Run at 6 AM, 12 PM, and 6 PM |

Complex patterns like "every third Tuesday" or "first Monday of each month" are automatically handled via AI parsing.

## Timezone Support

Specify timezones using common abbreviations or full IANA names:

- **EST/EDT** → America/New_York
- **CST/CDT** → America/Chicago
- **MST/MDT** → America/Denver
- **PST/PDT** → America/Los_Angeles
- **UTC** → UTC
- **IANA names** → e.g., "Europe/London", "Asia/Tokyo"

Default timezone is UTC if not specified.

## Example Conversations

### Basic Interval

**You:** Check my cluster every 30 minutes

**System:**
```
Schedule configured for agent 'k8s-monitor':

Cron expression: 0 */30 * * * *
Timezone: UTC

Next 3 scheduled runs:
1. Fri Feb 14, 2026 at 06:30 PM UTC
2. Fri Feb 14, 2026 at 07:00 PM UTC
3. Fri Feb 14, 2026 at 07:30 PM UTC

Review the generated triggers.yaml configuration.
```

### Daily with Timezone

**You:** Run incident triage daily at 6am EST

**System:**
```
Schedule configured for agent 'incident-responder':

Cron expression: 0 0 6 * * *
Timezone: America/New_York

Next 3 scheduled runs:
1. Sat Feb 15, 2026 at 06:00 AM EST
2. Sun Feb 16, 2026 at 06:00 AM EST
3. Mon Feb 17, 2026 at 06:00 AM EST

Review the generated triggers.yaml configuration.
```

### Business Hours

**You:** Monitor Postgres performance during business hours

**System:**
```
Schedule configured for agent 'postgres-monitor':

Cron expression: 0 0 9-17 * * 1-5
Timezone: UTC

Next 3 scheduled runs:
1. Mon Feb 17, 2026 at 09:00 AM UTC
2. Mon Feb 17, 2026 at 10:00 AM UTC
3. Mon Feb 17, 2026 at 11:00 AM UTC

Review the generated triggers.yaml configuration.
```

### Complex Pattern

**You:** Check deployments every third Tuesday at 2pm PST

**System:**
```
Schedule configured for agent 'deployment-checker':

Cron expression: 0 0 14 * * 2#3
Timezone: America/Los_Angeles

Next 3 scheduled runs:
1. Tue Feb 18, 2026 at 02:00 PM PST
2. Tue Mar 18, 2026 at 02:00 PM PST
3. Tue Apr 15, 2026 at 02:00 PM PST

Review the generated triggers.yaml configuration.
```

## What Gets Generated

The system generates a `triggers.yaml` snippet you can add to your AOF configuration:

```yaml
# Trigger configuration for k8s-monitor
# Add this to your triggers.yaml file

schedules:
  - id: k8s-monitor-schedule
    agent_id: k8s-monitor
    trigger:
      type: Schedule
      schedule: "0 */30 * * * *"
      timezone: "UTC"
    description: "Check my cluster every 30 minutes"
```

### Field Explanations

- **id** - Unique identifier for this schedule (auto-generated)
- **agent_id** - Which agent to execute
- **schedule** - 6-field cron expression (second, minute, hour, day, month, day-of-week)
- **timezone** - IANA timezone string for schedule interpretation
- **description** - Human-readable description of the schedule

## Verifying Your Schedule

### Next Run Times

Before confirming, the system shows you the next 3 scheduled runs. This helps catch:

- **Timezone mistakes** - "6am EST" vs "6am PST" is a 3-hour difference
- **Day-of-week errors** - "every weekday" vs "every day"
- **Frequency misunderstandings** - "every 30 minutes" vs "30 minutes past the hour"

### Modifying a Schedule

If the next runs don't match your intent:
1. Cancel the current configuration
2. Rephrase your schedule description
3. System will generate a new cron expression

## Integration with `aofctl serve`

Once you've confirmed the schedule:

1. **Save triggers.yaml** - Add the generated snippet to your `triggers.yaml` file
2. **Start the daemon** - Run `aofctl serve --trigger-config triggers.yaml`
3. **Verify scheduling** - Check logs to see scheduled runs

The daemon will:
- Load your schedule configuration at startup
- Execute the agent at the specified times
- Respect timezone conversions (handles DST automatically)
- Log each scheduled execution

## Tips

- **Be specific about timezones** - Always include timezone if your schedule is time-sensitive
- **Check next runs** - The 3 upcoming times help verify your schedule is correct
- **Use common patterns** - "every N minutes/hours" is simpler than complex cron
- **Combine patterns** - "every weekday at 9am EST" combines day-of-week with time and timezone
