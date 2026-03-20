import { type ComponentProps } from "react";
import { Button } from "@/components/ui/8bit/button";
import { cn } from "@/lib/utils";

const colorStyles = {
  sand: {
    hover: "hover:!bg-sand hover:!text-background",
    active: "!bg-sand !text-background",
  },
  blood: {
    hover: "hover:!bg-blood hover:!text-background",
    active: "!bg-blood !text-background",
  },
} as const;

type ButtonColor = keyof typeof colorStyles;

interface ColorButtonProps extends ComponentProps<typeof Button> {
  /** Button accent color (default: "sand") */
  color?: ButtonColor;
  /** Force the active (highlighted) state programmatically */
  active?: boolean;
}

/**
 * Outline button with transparent background and colored hover.
 * Use `active` prop to force the highlighted state (e.g. for menu selection).
 */
export function ColorButton({
  color = "sand",
  active,
  className,
  ...props
}: ColorButtonProps) {
  const styles = colorStyles[color];

  return (
    <Button
      variant="outline"
      className={cn(
        "!bg-transparent text-gray-400",
        styles.hover,
        active && styles.active,
        className,
      )}
      {...props}
    />
  );
}
