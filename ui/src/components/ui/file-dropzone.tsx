import * as React from "react";
import { useDropzone } from "react-dropzone";
import { Upload, File, X } from "lucide-react";
import { cn } from "@/utils/cn";
import { Button } from "./button";

interface FileDropzoneProps {
  onDrop: (files: File[]) => void;
  accept?: string | string[];
  multiple?: boolean;
  maxFiles?: number;
  maxSize?: number;
  disabled?: boolean;
  className?: string;
  children?: React.ReactNode;
}

export const FileDropzone = React.forwardRef<HTMLDivElement, FileDropzoneProps>(
  ({
    onDrop,
    accept,
    multiple = true,
    maxFiles,
    maxSize,
    disabled = false,
    className,
    children,
    ...props
  }, ref) => {
    const {
      getRootProps,
      getInputProps,
      isDragActive,
      isDragReject,
      fileRejections,
      acceptedFiles,
    } = useDropzone({
      onDrop,
      accept: accept ? (Array.isArray(accept) ? accept.reduce((acc, type) => ({ ...acc, [type]: [] }), {}) : { [accept]: [] }) : undefined,
      multiple,
      maxFiles,
      maxSize,
      disabled,
    });

    const formatFileSize = (bytes: number) => {
      if (bytes === 0) return '0 Bytes';
      const k = 1024;
      const sizes = ['Bytes', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    return (
      <div className={cn("w-full", className)}>
        <div
          ref={ref}
          {...getRootProps()}
          className={cn(
            "border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors",
            isDragActive && !isDragReject && "border-primary bg-primary/5",
            isDragReject && "border-red-500 bg-red-50",
            disabled && "opacity-50 cursor-not-allowed",
            "hover:border-primary/50"
          )}
          {...props}
        >
          <input {...getInputProps()} />
          
          {children || (
            <div className="flex flex-col items-center gap-4">
              <div className="p-3 rounded-full bg-muted">
                <Upload className="h-6 w-6 text-muted-foreground" />
              </div>
              <div>
                <p className="text-sm font-medium">
                  {isDragActive
                    ? isDragReject
                      ? "Some files are not accepted"
                      : "Drop files here"
                    : "Drag & drop files here, or click to select"}
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  {accept && `Accepted types: ${Array.isArray(accept) ? accept.join(', ') : accept}`}
                  {maxSize && ` • Max size: ${formatFileSize(maxSize)}`}
                  {maxFiles && ` • Max files: ${maxFiles}`}
                </p>
              </div>
            </div>
          )}
        </div>

        {fileRejections.length > 0 && (
          <div className="mt-4 space-y-2">
            <p className="text-sm font-medium text-red-600">Rejected files:</p>
            {fileRejections.map(({ file, errors }) => (
              <div key={file.name} className="flex items-center justify-between p-2 bg-red-50 rounded">
                <div className="flex items-center gap-2">
                  <File className="h-4 w-4 text-red-500" />
                  <span className="text-sm text-red-700">{file.name}</span>
                </div>
                <div className="text-xs text-red-600">
                  {errors.map(e => e.message).join(', ')}
                </div>
              </div>
            ))}
          </div>
        )}

        {acceptedFiles.length > 0 && (
          <div className="mt-4 space-y-2">
            <p className="text-sm font-medium">Selected files:</p>
            {acceptedFiles.map((file) => (
              <div key={file.name} className="flex items-center justify-between p-2 bg-muted rounded">
                <div className="flex items-center gap-2">
                  <File className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm">{file.name}</span>
                  <span className="text-xs text-muted-foreground">
                    ({formatFileSize(file.size)})
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }
);

FileDropzone.displayName = "FileDropzone";
