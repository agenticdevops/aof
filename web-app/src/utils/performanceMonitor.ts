/**
 * Performance monitoring utilities for tracking latencies
 *
 * Tracks:
 * - WebSocket event receive → Redux dispatch latency (target <100ms)
 * - Redux dispatch → React render latency (target <50ms)
 * - Total event → UI update latency (target <150ms)
 */

/**
 * Performance metrics for a single event
 */
export interface PerformanceMetrics {
  eventType: string
  receiveTime: number
  dispatchTime?: number
  renderTime?: number
  totalLatency?: number
}

/**
 * Performance monitor singleton
 */
class PerformanceMonitor {
  private static instance: PerformanceMonitor | null = null
  private metrics: Map<string, PerformanceMetrics> = new Map()
  private readonly maxMetrics = 100
  private enabled = true

  // Thresholds (in ms)
  private readonly WEBSOCKET_THRESHOLD = 100
  private readonly RENDER_THRESHOLD = 50
  private readonly TOTAL_THRESHOLD = 150

  private constructor() {}

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor()
    }
    return PerformanceMonitor.instance
  }

  /**
   * Enable/disable performance monitoring
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled
  }

  /**
   * Start tracking an event
   */
  startEvent(eventId: string, eventType: string): void {
    if (!this.enabled) return

    this.metrics.set(eventId, {
      eventType,
      receiveTime: performance.now(),
    })

    // Clean up old metrics
    if (this.metrics.size > this.maxMetrics) {
      const oldestKeys = Array.from(this.metrics.keys()).slice(0, 20)
      oldestKeys.forEach(key => this.metrics.delete(key))
    }
  }

  /**
   * Record dispatch time
   */
  recordDispatch(eventId: string): void {
    if (!this.enabled) return

    const metric = this.metrics.get(eventId)
    if (metric) {
      metric.dispatchTime = performance.now()

      const latency = metric.dispatchTime - metric.receiveTime
      if (latency > this.WEBSOCKET_THRESHOLD) {
        console.warn(
          `[Performance] Slow WebSocket → Redux: ${latency.toFixed(2)}ms (${metric.eventType})`
        )
      }
    }
  }

  /**
   * Record render time
   */
  recordRender(eventId: string): void {
    if (!this.enabled) return

    const metric = this.metrics.get(eventId)
    if (metric && metric.dispatchTime) {
      metric.renderTime = performance.now()
      metric.totalLatency = metric.renderTime - metric.receiveTime

      const renderLatency = metric.renderTime - metric.dispatchTime
      if (renderLatency > this.RENDER_THRESHOLD) {
        console.warn(
          `[Performance] Slow Redux → Render: ${renderLatency.toFixed(2)}ms (${metric.eventType})`
        )
      }

      if (metric.totalLatency > this.TOTAL_THRESHOLD) {
        console.warn(
          `[Performance] Slow total latency: ${metric.totalLatency.toFixed(2)}ms (${metric.eventType})`
        )
      }
    }
  }

  /**
   * Get metrics for a specific event
   */
  getMetrics(eventId: string): PerformanceMetrics | undefined {
    return this.metrics.get(eventId)
  }

  /**
   * Get all metrics
   */
  getAllMetrics(): PerformanceMetrics[] {
    return Array.from(this.metrics.values())
  }

  /**
   * Get average latencies by event type
   */
  getAverageLatencies(): Map<string, { avg: number; count: number }> {
    const totals = new Map<string, { sum: number; count: number }>()

    this.metrics.forEach(metric => {
      if (metric.totalLatency) {
        const existing = totals.get(metric.eventType) || { sum: 0, count: 0 }
        totals.set(metric.eventType, {
          sum: existing.sum + metric.totalLatency,
          count: existing.count + 1,
        })
      }
    })

    const averages = new Map<string, { avg: number; count: number }>()
    totals.forEach((value, key) => {
      averages.set(key, {
        avg: value.sum / value.count,
        count: value.count,
      })
    })

    return averages
  }

  /**
   * Clear all metrics
   */
  clear(): void {
    this.metrics.clear()
  }

  /**
   * Log performance summary
   */
  logSummary(): void {
    if (!this.enabled) return

    console.group('[Performance Summary]')

    const averages = this.getAverageLatencies()
    if (averages.size === 0) {
      console.log('No metrics recorded')
    } else {
      console.table(
        Array.from(averages.entries()).map(([eventType, { avg, count }]) => ({
          'Event Type': eventType,
          'Avg Latency (ms)': avg.toFixed(2),
          'Count': count,
          'Status': avg < this.TOTAL_THRESHOLD ? '✓' : '⚠️',
        }))
      )
    }

    console.groupEnd()
  }
}

/**
 * Get performance monitor instance
 */
export function getPerformanceMonitor(): PerformanceMonitor {
  return PerformanceMonitor.getInstance()
}

/**
 * Convenience function to start tracking an event
 */
export function startPerformanceTracking(eventId: string, eventType: string): void {
  getPerformanceMonitor().startEvent(eventId, eventType)
}

/**
 * Convenience function to record dispatch time
 */
export function recordDispatchTime(eventId: string): void {
  getPerformanceMonitor().recordDispatch(eventId)
}

/**
 * Convenience function to record render time
 */
export function recordRenderTime(eventId: string): void {
  getPerformanceMonitor().recordRender(eventId)
}

/**
 * Enable/disable performance monitoring globally
 */
export function setPerformanceMonitoring(enabled: boolean): void {
  getPerformanceMonitor().setEnabled(enabled)
}

/**
 * Log performance summary
 */
export function logPerformanceSummary(): void {
  getPerformanceMonitor().logSummary()
}

// Export singleton for direct access
export const performanceMonitor = PerformanceMonitor.getInstance()
