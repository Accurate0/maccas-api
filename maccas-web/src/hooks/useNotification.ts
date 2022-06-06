import { useSnackbar, VariantType } from "notistack";
import { useEffect, useState } from "react";

interface NotificationConf {
  msg?: string;
  variant?: VariantType;
}

const useNotification = () => {
  const [conf, setConf] = useState<NotificationConf>({});
  const { enqueueSnackbar } = useSnackbar();

  useEffect(() => {
    if (conf?.msg) {
      enqueueSnackbar(conf.msg, {
        variant: conf.variant ?? ("default" as VariantType),
        autoHideDuration: 3000,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [conf]);
  return setConf;
};

export default useNotification;
