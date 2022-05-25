import { Button, Grid, TextField } from "@mui/material";
import { useState } from "react";
import useLocationSearch from "../hooks/useLocationSearch";
import { useUpdateUserConfig } from "../hooks/useUserConfig";

const LocationSelection = () => {
  const [value, setValue] = useState<string>();
  const { search } = useLocationSearch();
  const updateConfig = useUpdateUserConfig();

  const searchAndUpdate = async (text: string) => {
    const resp = await search(text);
    await updateConfig({ storeId: resp.storeNumber.toString(), storeName: resp.name });
  };

  return (
    <Grid
      container
      spacing={0}
      direction="column"
      alignItems="center"
      justifyContent="center"
      style={{ minHeight: "100vh" }}
    >
      <Grid item xs={3} paddingBottom={2}>
        <TextField label="Location" value={value} onChange={(e) => setValue(e.target.value)} />
      </Grid>
      <Grid item>
        <Button
          variant="contained"
          onClick={() => {
            searchAndUpdate(value ?? "");
          }}
        >
          Search
        </Button>
      </Grid>
    </Grid>
  );
};

export default LocationSelection;
