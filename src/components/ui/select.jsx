import * as React from "react"
import { cn } from "../../lib/utils"
import { ChevronDown, Check } from "lucide-react"

// Simplified Select implementation without Radix UI
// This is a custom naive implementation to match the API surface used in App.jsx

const SelectContext = React.createContext({})

const Select = ({ value, onValueChange, children }) => {
    const [open, setOpen] = React.useState(false);
    return (
        <SelectContext.Provider value={{ value, onValueChange, open, setOpen }}>
            <div className="relative w-full">{children}</div>
        </SelectContext.Provider>
    )
}

const SelectTrigger = React.forwardRef(({ className, children, ...props }, ref) => {
    const { open, setOpen } = React.useContext(SelectContext)
    return (
        <button
            ref={ref}
            type="button"
            onClick={() => setOpen(!open)}
            className={cn(
                "flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
                className
            )}
            {...props}
        >
            {children}
            <ChevronDown className="h-4 w-4 opacity-50" />
        </button>
    )
})
SelectTrigger.displayName = "SelectTrigger"

const SelectValue = React.forwardRef(({ className, ...props }, ref) => {
    const { value } = React.useContext(SelectContext)
    return (
        <span
            ref={ref}
            className={cn("block truncate", className)}
            {...props}
        >
            {value || props.placeholder}
        </span>
    )
})
SelectValue.displayName = "SelectValue"

const SelectContent = React.forwardRef(({ className, children, position = "popper", ...props }, ref) => {
    const { open } = React.useContext(SelectContext)

    if (!open) return null

    return (
        <div
            ref={ref}
            className={cn(
                "absolute z-50 min-w-[8rem] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md animate-in fade-in-80 w-full mt-1",
                position === "popper" && "translate-y-1",
                className
            )}
            {...props}
        >
            <div className="p-1">
                {children}
            </div>
        </div>
    )
})
SelectContent.displayName = "SelectContent"

const SelectItem = React.forwardRef(({ className, children, value, ...props }, ref) => {
    const { value: selectedValue, onValueChange, setOpen } = React.useContext(SelectContext)
    const isSelected = selectedValue === value

    return (
        <div
            ref={ref}
            className={cn(
                "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
                isSelected && "bg-accent text-accent-foreground",
                className
            )}
            onClick={() => {
                onValueChange(value)
                setOpen(false)
            }}
            {...props}
        >
            <span className="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                {isSelected && <Check className="h-4 w-4" />}
            </span>
            {children}
        </div>
    )
})
SelectItem.displayName = "SelectItem"

const SelectGroup = React.Fragment
const SelectLabel = React.Fragment
const SelectSeparator = React.Fragment
const SelectScrollUpButton = React.Fragment
const SelectScrollDownButton = React.Fragment

export { Select, SelectGroup, SelectValue, SelectTrigger, SelectContent, SelectLabel, SelectItem, SelectSeparator, SelectScrollUpButton, SelectScrollDownButton }
