import * as React from "react";
import { Copy, Download } from "lucide-react";
import { cn } from "@/utils/cn";
import { Button } from "./button";

interface CodeViewerProps {
  code: string;
  language?: string;
  theme?: 'light' | 'dark';
  readOnly?: boolean;
  showLineNumbers?: boolean;
  wrapLines?: boolean;
  maxHeight?: string;
  className?: string;
}

export const CodeViewer = React.forwardRef<HTMLDivElement, CodeViewerProps>(
  ({ 
    code, 
    language = 'text', 
    theme = 'light', 
    readOnly = true, 
    showLineNumbers = true, 
    wrapLines = false,
    maxHeight = '400px',
    className,
    ...props 
  }, ref) => {
    const [copied, setCopied] = React.useState(false);

    const handleCopy = async () => {
      try {
        await navigator.clipboard.writeText(code);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (err) {
        console.error('Failed to copy code:', err);
      }
    };

    const handleDownload = () => {
      const blob = new Blob([code], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `code.${language === 'json' ? 'json' : 'txt'}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    };

    return (
      <div
        ref={ref}
        className={cn(
          "relative rounded-lg border bg-background",
          className
        )}
        {...props}
      >
        <div className="flex items-center justify-between border-b px-4 py-2">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-muted-foreground">
              {language.toUpperCase()}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCopy}
              className="h-8 w-8 p-0"
            >
              <Copy className="h-4 w-4" />
              <span className="sr-only">Copy code</span>
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleDownload}
              className="h-8 w-8 p-0"
            >
              <Download className="h-4 w-4" />
              <span className="sr-only">Download code</span>
            </Button>
          </div>
        </div>
        <div
          className="overflow-auto p-4 font-mono text-sm"
          style={{ maxHeight }}
        >
          <pre
            className={cn(
              "whitespace-pre",
              wrapLines ? "whitespace-pre-wrap" : "whitespace-pre",
              showLineNumbers && "pl-8"
            )}
          >
            {code}
          </pre>
        </div>
        {copied && (
          <div className="absolute top-2 right-2 rounded bg-green-100 px-2 py-1 text-xs text-green-800">
            Copied!
          </div>
        )}
      </div>
    );
  }
);

CodeViewer.displayName = "CodeViewer";
