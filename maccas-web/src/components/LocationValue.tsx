import { Button, Grid, Typography } from "@mui/material";
import { useNavigate } from "react-router-dom";
import useUserConfig from "../hooks/useUserConfig";

const LocationValue = () => {
  const { config } = useUserConfig();
  const navigate = useNavigate();

  return (
    <>
      <Grid container spacing={2}>
        <Grid item alignSelf="center">
          <Button color="inherit" onClick={() => navigate("/location")}>
            <Typography>Store: {config?.storeName || "None"}</Typography>
          </Button>
        </Grid>
      </Grid>
    </>
  );
};

export default LocationValue;
