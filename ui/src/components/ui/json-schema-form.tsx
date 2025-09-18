import * as React from "react";
import Form from "@rjsf/core";
import { RJSFSchema, UiSchema } from "@rjsf/utils";
import { cn } from "@/utils/cn";
import { Button } from "./button";

interface JsonSchemaFormProps {
  schema: RJSFSchema;
  uiSchema?: UiSchema;
  formData?: any;
  onChange?: (data: any) => void;
  onSubmit?: (data: any) => void;
  onError?: (errors: any) => void;
  disabled?: boolean;
  readonly?: boolean;
  liveValidate?: boolean;
  showErrorList?: boolean;
  className?: string;
}

export const JsonSchemaForm = React.forwardRef<HTMLFormElement, JsonSchemaFormProps>(
  ({
    schema,
    uiSchema,
    formData,
    onChange,
    onSubmit,
    onError,
    disabled = false,
    readonly = false,
    liveValidate = true,
    showErrorList = true,
    className,
    ...props
  }, ref) => {
    const [data, setData] = React.useState(formData || {});
    const [errors, setErrors] = React.useState<any[]>([]);

    const handleChange = (event: any) => {
      const newData = event.formData;
      setData(newData);
      onChange?.(newData);
    };

    const handleSubmit = (event: any) => {
      if (event.errors.length === 0) {
        onSubmit?.(event.formData);
      } else {
        onError?.(event.errors);
      }
    };

    const handleError = (errors: any) => {
      setErrors(errors);
      onError?.(errors);
    };

    return (
      <div className={cn("w-full", className)}>
        <Form
          ref={ref}
          schema={schema}
          uiSchema={uiSchema}
          formData={data}
          onChange={handleChange}
          onSubmit={handleSubmit}
          onError={handleError}
          disabled={disabled}
          readonly={readonly}
          liveValidate={liveValidate}
          showErrorList={showErrorList}
          {...props}
        >
          <div className="flex justify-end gap-2 mt-4">
            <Button type="submit" disabled={disabled || errors.length > 0}>
              Submit
            </Button>
          </div>
        </Form>
      </div>
    );
  }
);

JsonSchemaForm.displayName = "JsonSchemaForm";
