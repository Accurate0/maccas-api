import { Button, Dialog, DialogTitle, Grid, IconButton, List, ListItem, ListItemText, TextField } from "@mui/material";
import { useState } from "react";
import useLocationSearch from "../hooks/useLocationSearch";
import { useUpdateUserConfig } from "../hooks/useUserConfig";
import MyLocationIcon from "@mui/icons-material/MyLocation";
import useNotification from "../hooks/useNotification";
import useLocations from "../hooks/useLocations";
import { Restaurant } from "../types";

const options: PositionOptions = {
  enableHighAccuracy: true,
  timeout: 5000,
  maximumAge: 0,
};

const LocationSelection = () => {
  const [dialogOpen, setDialogOpen] = useState<boolean>(false);
  const [restaurants, setRestaurants] = useState<Restaurant[]>([]);
  const [value, setValue] = useState<string>();
  const [error, setError] = useState<boolean>();
  const { search } = useLocationSearch();
  const updateConfig = useUpdateUserConfig();
  const notification = useNotification();
  const { search: searchByPosition } = useLocations();

  const searchAndUpdate = async (text: string | undefined) => {
    const trimmedText = text?.trim();
    if (trimmedText) {
      const resp = await search(trimmedText);
      await updateConfig({ storeId: resp.storeNumber.toString(), storeName: resp.name });
      setError(false);
    } else {
      setError(true);
    }
  };

  const searchByLocation = async () => {
    if (navigator.geolocation) {
      const result = await navigator.permissions.query({ name: "geolocation" });
      switch (result.state) {
        case "granted":
        case "prompt":
          navigator.geolocation.getCurrentPosition(
            async (position) => {
              const response = await searchByPosition(position.coords.latitude, position.coords.longitude);
              if (response.response?.restaurants?.length) {
                setRestaurants(response.response.restaurants);
                setDialogOpen(true);
              } else {
                notification({ msg: "No locations found nearby", variant: "error" });
              }
            },
            (err) => notification({ msg: err.message, variant: "error" }),
            options
          );
          break;
        case "denied":
          notification({ msg: "Location access denied", variant: "error" });
          break;
      }
    } else {
      notification({ msg: "Location not available", variant: "error" });
    }
  };

  const handleClose = () => setDialogOpen(false);
  const handleListItemClick = (value: Restaurant) => {
    updateConfig({ storeId: value.nationalStoreNumber.toString(), storeName: value.name });
    handleClose();
  };

  return (
    <>
      <Dialog onClose={handleClose} open={dialogOpen}>
        <DialogTitle>Nearby Locations</DialogTitle>
        <List sx={{ pt: 0 }}>
          {restaurants.map((restaurant) => (
            <ListItem
              button
              onClick={() => handleListItemClick(restaurant)}
              key={restaurant.nationalStoreNumber.toString()}
            >
              <ListItemText primary={restaurant.name} secondary={restaurant.address.addressLine1} />
            </ListItem>
          ))}
        </List>
      </Dialog>
      <Grid
        container
        spacing={0}
        direction="column"
        alignItems="center"
        justifyContent="center"
        style={{ minHeight: "100vh" }}
      >
        <Grid item xs={3} paddingBottom={2} container justifyContent="center" alignItems="center">
          <Grid item>
            <TextField
              label="Location"
              value={value}
              helperText={error ? "Enter location" : undefined}
              error={error}
              type="text"
              onChange={(e) => setValue(e.target.value)}
              InputProps={{
                endAdornment: (
                  <IconButton size="large" onClick={searchByLocation}>
                    <MyLocationIcon />
                  </IconButton>
                ),
              }}
            />
          </Grid>
          <Grid item></Grid>
        </Grid>
        <Grid item>
          <Button
            variant="contained"
            onClick={() => {
              searchAndUpdate(value);
            }}
          >
            Search
          </Button>
        </Grid>
      </Grid>
    </>
  );
};

export default LocationSelection;
