import * as React from "react";
import { cn } from "@/utils/cn";

interface SplitPaneProps {
  children: [React.ReactNode, React.ReactNode];
  defaultSize?: number;
  minSize?: number;
  maxSize?: number;
  split?: 'vertical' | 'horizontal';
  resizable?: boolean;
  className?: string;
}

const SplitPane = React.forwardRef<HTMLDivElement, SplitPaneProps>(
  ({ 
    children, 
    defaultSize = 50, 
    minSize = 20, 
    maxSize = 80, 
    split = 'vertical', 
    resizable = true,
    className,
    ...props 
  }, ref) => {
    const [sizes, setSizes] = React.useState([defaultSize, 100 - defaultSize]);
    const [isDragging, setIsDragging] = React.useState(false);
    const containerRef = React.useRef<HTMLDivElement>(null);
    const startPosRef = React.useRef<number>(0);
    const startSizesRef = React.useRef<number[]>([]);

    const handleMouseDown = React.useCallback((e: React.MouseEvent) => {
      if (!resizable) return;
      
      e.preventDefault();
      setIsDragging(true);
      startPosRef.current = split === 'vertical' ? e.clientX : e.clientY;
      startSizesRef.current = [...sizes];
      
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    }, [resizable, split, sizes]);

    const handleMouseMove = React.useCallback((e: MouseEvent) => {
      if (!isDragging || !containerRef.current) return;
      
      const containerRect = containerRef.current.getBoundingClientRect();
      const containerSize = split === 'vertical' ? containerRect.width : containerRect.height;
      const currentPos = split === 'vertical' ? e.clientX : e.clientY;
      const startPos = startPosRef.current;
      const startSizes = startSizesRef.current;
      
      const delta = currentPos - startPos;
      const deltaPercent = (delta / containerSize) * 100;
      
      const newSize1 = Math.max(
        minSize,
        Math.min(maxSize, startSizes[0] + deltaPercent)
      );
      const newSize2 = 100 - newSize1;
      
      setSizes([newSize1, newSize2]);
    }, [isDragging, split, minSize, maxSize]);

    const handleMouseUp = React.useCallback(() => {
      setIsDragging(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    }, [handleMouseMove]);

    React.useEffect(() => {
      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };
    }, [handleMouseMove, handleMouseUp]);

    const containerClasses = cn(
      "flex w-full h-full",
      split === 'vertical' ? 'flex-row' : 'flex-col',
      className
    );

    const paneClasses = cn(
      "overflow-hidden",
      split === 'vertical' ? 'h-full' : 'w-full'
    );

    const dividerClasses = cn(
      "bg-border hover:bg-border/80 transition-colors",
      split === 'vertical' 
        ? 'w-1 cursor-col-resize hover:w-2' 
        : 'h-1 cursor-row-resize hover:h-2',
      isDragging && (split === 'vertical' ? 'w-2' : 'h-2'),
      resizable ? 'cursor-col-resize' : 'cursor-default'
    );

    return (
      <div
        ref={containerRef}
        className={containerClasses}
        {...props}
      >
        <div
          className={paneClasses}
          style={{
            [split === 'vertical' ? 'width' : 'height']: `${sizes[0]}%`,
          }}
        >
          {children[0]}
        </div>
        
        <div
          className={dividerClasses}
          onMouseDown={handleMouseDown}
          role="separator"
          aria-orientation={split}
          aria-valuenow={sizes[0]}
          aria-valuemin={minSize}
          aria-valuemax={maxSize}
        />
        
        <div
          className={paneClasses}
          style={{
            [split === 'vertical' ? 'width' : 'height']: `${sizes[1]}%`,
          }}
        >
          {children[1]}
        </div>
      </div>
    );
  }
);

SplitPane.displayName = "SplitPane";

export { SplitPane };
