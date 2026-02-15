import React from 'react'
import { Search } from 'lucide-react'
import Input from './Input'

interface SearchBarProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type' | 'onChange'> {
  fullWidth?: boolean
  value?: string
  onChange?: (value: string) => void
}

export const SearchBar = React.forwardRef<HTMLInputElement, SearchBarProps>(
  ({ fullWidth = true, value, onChange, ...props }, ref) => {
    return (
      <Input
        ref={ref}
        type="text"
        icon={<Search className="w-5 h-5" />}
        iconPosition="left"
        placeholder="Search..."
        fullWidth={fullWidth}
        value={value}
        onChange={(e) => onChange?.(e.target.value)}
        {...(props as any)}
      />
    )
  }
)

SearchBar.displayName = 'SearchBar'
export default SearchBar
