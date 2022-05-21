import { Button, Grid, TextField } from "@mui/material";
import { useState } from "react";
import useLocationSearch from "../hooks/useLocationSearch";
import useUserConfig from "../hooks/useUserConfig";

const LocationSelection = () => {
  const [value, setValue] = useState<string>();
  const { search } = useLocationSearch();
  const { updateConfig } = useUserConfig();

  const searchAndUpdate = async (text: string) => {
    const resp = await search(text);
    await updateConfig({ storeId: resp.storeNumber.toString(), storeName: resp.name });
  };

  return (
    <>
      <Grid item paddingBottom={2}>
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
    </>
  );
};

export default LocationSelection;
