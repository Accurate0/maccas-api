import { Grid, FormControl, InputLabel, Select, MenuItem, Button } from "@mui/material";
import moment from "moment";
import { useState } from "react";
import useDeals from "../hooks/useDeals";
import { Offer } from "../types";

export interface DealSelectorProps {
  onSelection: (deal?: Offer) => void;
}

const DealSelector: React.FC<DealSelectorProps> = ({ onSelection }) => {
  const [selected, setSelected] = useState<string>();
  const deals = useDeals();

  const isOfferValid = (deal: Offer) => {
    const from = moment.utc(deal.validFromUTC);
    const to = moment.utc(deal.localValidTo);
    const now = new Date();

    return moment.utc(now).isBetween(from, to);
  };

  return (
    <>
      <Grid item>
        <FormControl sx={{ m: 1, minWidth: 300 }}>
          <InputLabel>Deal</InputLabel>
          <Select label="Deal" onChange={(e) => setSelected(e.target.value as string)}>
            {deals?.map((o) => (
              <MenuItem value={o.dealUuid ?? ""}>
                {isOfferValid(o) ? "✅" : "❌"} {o.name.split("\n")[0]} ({o.count})
              </MenuItem>
            ))}
          </Select>
        </FormControl>
      </Grid>
      <Grid item>
        <Button
          style={{ visibility: !!!selected ? "hidden" : undefined }}
          variant="contained"
          onClick={() => onSelection(deals?.find((d) => d.dealUuid === selected))}
        >
          Next
        </Button>
      </Grid>
    </>
  );
};

export default DealSelector;
