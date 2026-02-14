/**
 * PersonalityTraits component - renders personality trait badges.
 * Displays up to 3 traits as colored pill badges with tooltips.
 * Color-coded by trait category for visual distinction.
 */

import React, { useState } from 'react';

/**
 * Component props.
 */
export interface PersonalityTraitsProps {
  /** Array of personality trait strings (e.g., ["methodical", "proactive", "detail-oriented"]) */
  traits: string[];

  /** Optional className for container styling */
  className?: string;
}

/**
 * Maximum number of traits to display before showing "+N more".
 */
const MAX_VISIBLE_TRAITS = 3;

/**
 * Trait category color mapping.
 * Groups traits by theme for consistent visual coding.
 */
const TRAIT_COLORS: Record<string, string> = {
  // Analytical / methodical traits (blue)
  methodical: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  proactive: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  'detail-oriented': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  systematic: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  analytical: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',

  // Investigative / curious traits (purple)
  curious: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  patient: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  thorough: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  persistent: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  investigative: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',

  // Leadership / calm traits (green)
  calm: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  decisive: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  communicative: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  confident: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  organized: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
};

/**
 * Default color for traits not in the predefined map.
 */
const DEFAULT_TRAIT_COLOR =
  'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';

/**
 * Get color classes for a given trait.
 */
function getTraitColor(trait: string): string {
  const normalized = trait.toLowerCase().trim();
  return TRAIT_COLORS[normalized] || DEFAULT_TRAIT_COLOR;
}

/**
 * PersonalityTraits component.
 *
 * @example
 * ```tsx
 * <PersonalityTraits traits={["methodical", "proactive", "detail-oriented"]} />
 * ```
 */
export function PersonalityTraits({
  traits,
  className = '',
}: PersonalityTraitsProps): React.ReactElement | null {
  const [tooltipTrait, setTooltipTrait] = useState<string | null>(null);
  const [showAll, setShowAll] = useState(false);

  // Don't render anything if no traits
  if (!traits || traits.length === 0) {
    return null;
  }

  const visibleTraits = showAll ? traits : traits.slice(0, MAX_VISIBLE_TRAITS);
  const hiddenCount = traits.length - MAX_VISIBLE_TRAITS;

  return (
    <div
      className={`flex flex-wrap gap-1.5 items-center ${className}`}
      role="list"
      aria-label="Personality traits"
    >
      {visibleTraits.map((trait) => {
        const truncated = trait.length > 20 ? `${trait.slice(0, 17)}...` : trait;

        return (
          <div key={trait} className="relative">
            <button
              type="button"
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium transition-colors cursor-pointer ${getTraitColor(trait)}`}
              role="listitem"
              aria-label={`Trait: ${trait}`}
              onMouseEnter={() => setTooltipTrait(trait)}
              onMouseLeave={() => setTooltipTrait(null)}
              onFocus={() => setTooltipTrait(trait)}
              onBlur={() => setTooltipTrait(null)}
            >
              {truncated}
            </button>

            {/* Tooltip */}
            {tooltipTrait === trait && (
              <div
                className="absolute z-20 bottom-full left-1/2 transform -translate-x-1/2 mb-1 px-2 py-1 bg-gray-900 text-white text-xs rounded shadow-lg whitespace-nowrap"
                role="tooltip"
              >
                This agent is {trait}
                <div className="absolute left-1/2 transform -translate-x-1/2 top-full w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900" />
              </div>
            )}
          </div>
        );
      })}

      {/* "+N more" link */}
      {hiddenCount > 0 && !showAll && (
        <button
          type="button"
          className="inline-flex items-center px-2 py-0.5 text-xs font-medium text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 cursor-pointer"
          onClick={() => setShowAll(true)}
          aria-label={`Show ${hiddenCount} more traits`}
        >
          +{hiddenCount} more
        </button>
      )}
    </div>
  );
}
