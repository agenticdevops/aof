import {
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
  closestCorners,
} from '@dnd-kit/core';

/**
 * Custom keyboard sensor configuration for accessible drag-and-drop.
 * Supports arrow keys for navigation and Space/Enter for drag actions.
 */
export const keyboardSensorOptions = {
  coordinateGetter: (event: KeyboardEvent) => {
    // Prevent default scrolling
    event.preventDefault();
    return undefined;
  },
};

/**
 * Collision detection algorithm for drag-and-drop.
 * Uses closestCorners for better UX when dragging near multiple drop targets.
 */
export const collisionDetectionAlgorithm = closestCorners;

/**
 * Hook that returns configured DnD sensors for mouse, touch, and keyboard interactions.
 *
 * Usage:
 * ```tsx
 * const sensors = useDndSensors();
 * <DndContext sensors={sensors}>...</DndContext>
 * ```
 */
export function useDndSensors() {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // 8px movement required to start drag (prevents accidental drags on click)
      },
    }),
    useSensor(TouchSensor, {
      activationConstraint: {
        delay: 250, // 250ms hold required on touch devices
        tolerance: 5, // 5px movement tolerance during delay
      },
    }),
    useSensor(KeyboardSensor, keyboardSensorOptions),
  );

  return sensors;
}

/**
 * Type guard to check if drag event has valid destination.
 */
export function hasValidDestination(event: DragEndEvent): boolean {
  return event.over !== null;
}

/**
 * Extract task ID from drag event active element.
 */
export function getTaskIdFromEvent(event: DragEndEvent): string {
  return String(event.active.id);
}

/**
 * Extract destination lane ID from drag event.
 */
export function getDestinationLaneFromEvent(event: DragEndEvent): string | null {
  if (!event.over) return null;
  return String(event.over.id);
}
