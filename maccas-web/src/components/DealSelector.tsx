import { Grid, Button, CardActions, CardContent, Typography, Paper, useMediaQuery } from "@mui/material";
import moment from "moment";
import useDeals from "../hooks/useDeals";
import { theme } from "../styles";
import { Offer } from "../types";

export interface DealSelectorProps {
  onSelection: (deal?: Offer) => void;
}

const DealSelector: React.FC<DealSelectorProps> = ({ onSelection }) => {
  const deals = useDeals();
  const mediaQuery = useMediaQuery(theme.breakpoints.down("md"));

  console.log(mediaQuery);

  const isOfferValid = (deal: Offer) => {
    const from = moment.utc(deal.validFromUTC);
    const to = moment.utc(deal.validToUTC);
    const now = new Date();

    return moment.utc(now).isBetween(from, to);
  };

  return (
    <>
      <Grid container spacing={2} paddingTop={4}>
        {deals?.map((o) => (
          <Grid item xs={6} md={4}>
            <Paper square>
              <CardContent style={{ height: mediaQuery ? "200px" : "120px", padding: "25px 25px 25px 25px" }}>
                <Typography variant="h6" component="div">
                  {o.name.split("\n")[0]}
                </Typography>
                <br />
                <Typography variant="body2">Added: {new Date(o.CreationDateUtc).toLocaleDateString()}</Typography>
                <br />
                <Typography sx={{ mb: 1.5 }} color="text.secondary">
                  <Grid container item spacing={2}>
                    <Grid item xs={3} md={1} style={{ color: theme.palette.text.primary }}>
                      {isOfferValid(o) ? "✅" : "❌"}
                    </Grid>
                    <Grid item xs={9}>
                      <Typography variant="caption">{o.count} available</Typography>
                    </Grid>
                  </Grid>
                </Typography>
                <br />
              </CardContent>
              <CardActions style={{ padding: "25px 25px 25px 25px" }}>
                <Button color="secondary" onClick={() => onSelection(o)}>
                  Select
                </Button>
              </CardActions>
            </Paper>
          </Grid>
        ))}
      </Grid>
    </>
  );
};

export default DealSelector;
