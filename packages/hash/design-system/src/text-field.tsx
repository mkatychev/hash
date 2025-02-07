import {
  faCheckCircle,
  faCircleExclamation,
} from "@fortawesome/free-solid-svg-icons";
import {
  Box,
  Collapse,
  formHelperTextClasses,
  InputAdornment,
  InputProps,
  outlinedInputClasses,
  TextField as MuiTextField,
  TextFieldProps as MuiTextFieldProps,
  Typography,
} from "@mui/material";
import { forwardRef, FunctionComponent, useState } from "react";
import { FontAwesomeIcon } from "./fontawesome-icon";

type TextFieldProps = {
  success?: boolean;
  showLabelCornerHint?: boolean;
  autoResize?: boolean;
} & MuiTextFieldProps;

/**
 * 'Freezes' a value when it's falsy, meaning the value will never update to
 * be falsy. Useful for keeping a component the same when animating out
 */
const useFrozenValue = <T extends any>(value: T): T => {
  const [frozenValue, setFrozenValue] = useState(value);

  if (value && frozenValue !== value) {
    setFrozenValue(value);
  }
  return frozenValue;
};

export const getInputProps = ({
  success,
  error,
  multiline,
  autoResize,
  ...otherProps
}: InputProps &
  Pick<
    TextFieldProps,
    "success" | "error" | "multiline" | "autoResize"
  > = {}): InputProps => {
  const { sx: InputPropsSx = [], ...otherInputProps } = otherProps;

  const renderEndAdornment = () => {
    if (error || success) {
      return (
        <InputAdornment position="end">
          <FontAwesomeIcon
            icon={success ? faCheckCircle : faCircleExclamation}
            sx={({ palette }) => ({
              color: success ? palette.green[60] : palette.red[60],
            })}
          />
        </InputAdornment>
      );
    }
    return null;
  };

  return {
    sx: [
      ({ palette }) => ({
        [`& .${outlinedInputClasses.notchedOutline}, &:hover .${outlinedInputClasses.notchedOutline}`]:
          {
            borderColor: success
              ? palette.green[60]
              : error
              ? palette.red[40]
              : palette.gray[30],
          },
        ...(multiline &&
          autoResize && {
            [`& .${outlinedInputClasses.input}`]: {
              resize: "auto",
            },
          }),
      }),
      ...(Array.isArray(InputPropsSx) ? InputPropsSx : [InputPropsSx]),
    ],
    ...{ notched: false },
    ...otherInputProps,
    endAdornment:
      error || success ? renderEndAdornment() : otherProps?.endAdornment,
  };
};

export const inputLabelProps = {
  disableAnimation: true,
  shrink: true,
};

export const TextField: FunctionComponent<TextFieldProps> = forwardRef(
  (
    {
      helperText,
      sx,
      InputProps: inputProps = {},
      success,
      error,
      label,
      showLabelCornerHint,
      autoResize,
      ...textFieldProps
    },
    ref,
  ) => {
    const frozenHelperText = useFrozenValue(helperText);

    return (
      <MuiTextField
        ref={ref}
        sx={[
          {
            ...(!helperText && {
              [`.${formHelperTextClasses.root}`]: {
                marginTop: 0,
              },
            }),
          },
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
        {...textFieldProps}
        error={error}
        label={
          label ? (
            <>
              {label}
              {showLabelCornerHint && (
                <Typography
                  component="span"
                  variant="smallTextLabels"
                  sx={({ palette }) => ({
                    position: "absolute",
                    right: 0,
                    fontWeight: 400,
                    color: palette.gray[60],
                  })}
                >
                  {textFieldProps.required ? "Required" : "Optional"}
                </Typography>
              )}
            </>
          ) : null
        }
        InputLabelProps={inputLabelProps}
        InputProps={getInputProps({
          ...inputProps,
          success,
          error,
          autoResize,
          multiline: textFieldProps.multiline,
        })}
        helperText={
          <Collapse in={!!helperText}>
            <Box>{frozenHelperText}</Box>
          </Collapse>
        }
        FormHelperTextProps={{
          ...{ as: "div" },
          error,
          sx: ({ typography, palette }) => ({
            marginLeft: 0,
            mt: 0.75,
            ...typography.smallTextLabels,
            color: error ? palette.red[80] : palette.gray[60],
          }),
        }}
        minRows={3}
      />
    );
  },
);

TextField.displayName = "TextField";
