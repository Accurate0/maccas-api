import { AuthenticatedTemplate, UnauthenticatedTemplate, useMsal } from "@azure/msal-react";
import { AppBar, Box, Button, Container, Grid, Toolbar, Typography } from "@mui/material";
import { useState } from "react";
import { Link, Route, Routes, useNavigate } from "react-router-dom";
import DealSelection from "./components/DealSelection";
import DealSelector from "./components/DealSelector";
import LocationSelection from "./components/LocationSelection";
import LocationValue from "./components/LocationValue";
import { LoginRequest } from "./config/msal";
import { Offer } from "./types";

const App = () => {
  const { instance } = useMsal();
  const navigate = useNavigate();
  const [deal, setDeal] = useState<Offer>();
  const onDealSelected = async (deal?: Offer) => {
    setDeal(deal);
    navigate("/code");
  };

  return (
    <>
      <UnauthenticatedTemplate>
        <Grid
          container
          spacing={0}
          direction="column"
          alignItems="center"
          justifyContent="center"
          style={{ minHeight: "100vh" }}
        >
          <Grid item xs={3}>
            <Button variant="contained" onClick={() => instance.loginRedirect(LoginRequest)}>
              Login
            </Button>
          </Grid>
        </Grid>
      </UnauthenticatedTemplate>
      <AuthenticatedTemplate>
        <Box sx={{ flexGrow: 1 }}>
          <AppBar position="fixed" color="primary" elevation={1}>
            <Toolbar variant="dense">
              <Grid justifyContent="space-between" container>
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
                    <Grid item>
                      <Button color="inherit" onClick={() => instance.logoutRedirect()}>
                        Logout
                      </Button>
                    </Grid>
                  </Grid>
                </Grid>
              </Grid>
            </Toolbar>
          </AppBar>
        </Box>
        <Container style={{ paddingTop: 50, paddingBottom: 50 }}>
          <Routes>
            <Route path="/" element={<DealSelector onSelection={onDealSelected} />} />
            <Route path="/code" element={<DealSelection selected={deal} />} />
            <Route path="/location" element={<LocationSelection />} />
          </Routes>
        </Container>
      </AuthenticatedTemplate>
    </>
  );
};

export default App;
