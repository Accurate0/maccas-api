import { Button, Grid, Typography } from "@mui/material";
import { useNavigate } from "react-router-dom";
import useUserConfig from "../hooks/useUserConfig";

const LocationValue = () => {
  const { config } = useUserConfig();
  const navigate = useNavigate();

  return (
    <>
      <Grid container spacing={2}>
        <Grid item alignSelf="center" alignItems="center">
          <Button color="inherit" onClick={() => navigate("/location")}>
            <Typography variant="caption">
              <b>Store: {config?.storeName || "None"}</b>
            </Typography>
          </Button>
        </Grid>
      </Grid>
    </>
  );
};

export default LocationValue;
