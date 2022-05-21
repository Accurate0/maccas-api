import { Grid, Typography } from "@mui/material";
import useUserConfig from "../hooks/useUserConfig";

const LocationValue = () => {
  const { config } = useUserConfig();

  return (
    <>
      {config && (
        <Grid container spacing={2}>
          <Grid item alignSelf="center">
            <Typography>Store: {config.storeName === "" ? "No Store Selected" : config.storeName}</Typography>
          </Grid>
        </Grid>
      )}
    </>
  );
};

export default LocationValue;
