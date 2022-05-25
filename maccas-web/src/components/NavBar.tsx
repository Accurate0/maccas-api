import { useMsal } from "@azure/msal-react";
import { Box, AppBar, Toolbar, Grid, Typography, Button } from "@mui/material";
import { Link } from "react-router-dom";
import LocationValue from "./LocationValue";

const NavBar = () => {
  const { instance } = useMsal();

  return (
    <Box sx={{ flexGrow: 1 }}>
      <AppBar position="fixed" color="primary" elevation={1}>
        <Toolbar variant="dense">
          <Grid justifyContent="space-between" alignItems="baseline" container>
            <Grid item>
              <Typography variant="h6" color="inherit" component="div">
                <Link to="/" style={{ textDecoration: "none", color: "inherit" }}>
                  Maccas
                </Link>
              </Typography>
            </Grid>
            <Grid item>
              <Grid container spacing={3}>
                <Grid item>
                  <LocationValue />
                </Grid>
                <Grid item style={{ paddingLeft: 0 }}>
                  <Button color="inherit" onClick={() => instance.logoutRedirect()}>
                    <Typography variant="caption">Logout</Typography>
                  </Button>
                </Grid>
              </Grid>
            </Grid>
          </Grid>
        </Toolbar>
      </AppBar>
    </Box>
  );
};

export default NavBar;
