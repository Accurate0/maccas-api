import { SnackbarKey, useSnackbar, VariantType } from "notistack";
import IconButton from "@mui/material/IconButton";
import CloseIcon from "@mui/material/SvgIcon/SvgIcon";
import React, { Fragment, useEffect, useState } from "react";

interface NotificationConf {
  msg?: string;
  variant?: VariantType;
}

const useNotification = () => {
  const [conf, setConf] = useState<NotificationConf>({});
  const { enqueueSnackbar, closeSnackbar } = useSnackbar();
  const action = (key: SnackbarKey | undefined) => (
    <Fragment>
      <IconButton
        onClick={() => {
          closeSnackbar(key);
        }}
      >
        <CloseIcon />
      </IconButton>
    </Fragment>
  );
  useEffect(() => {
    if (conf?.msg) {
      enqueueSnackbar(conf.msg, {
        variant: conf.variant ?? ("default" as VariantType),
        autoHideDuration: 3000,
        action,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [conf]);
  return setConf;
};

export default useNotification;
