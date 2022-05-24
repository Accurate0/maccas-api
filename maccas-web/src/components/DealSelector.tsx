import { Grid, Button, CardActions, CardContent, Typography, useMediaQuery, CardMedia, Card } from "@mui/material";
import moment from "moment";
import { IMAGE_BUCKET_BASE } from "../config/api";
import useDeals from "../hooks/useDeals";
import { theme } from "../styles";
import { Offer } from "../types";

export interface DealSelectorProps {
  onSelection: (deal?: Offer) => void;
}

const DealSelector: React.FC<DealSelectorProps> = ({ onSelection }) => {
  const deals = useDeals();
  const mediaQuery = useMediaQuery(theme.breakpoints.down("md"));

  const isOfferValid = (deal: Offer) => {
    const from = moment.utc(deal.validFromUTC);
    const to = moment.utc(deal.validToUTC);
    const now = new Date();

    return moment.utc(now).isBetween(from, to);
  };

  return (
    <>
      <Grid container spacing={2} paddingTop={8} paddingBottom={4}>
        {deals?.map((o) => (
          <Grid item xs={6} md={3}>
            <Card square>
              <CardMedia component="img" image={`${IMAGE_BUCKET_BASE}/${o.imageBaseName}`} alt="green iguana" />
              <CardContent style={{ height: mediaQuery ? "170px" : "160px", padding: "25px 25px 25px 25px" }}>
                <Grid container direction="column" justifyContent="space-evenly" alignItems="flex-start" spacing={2}>
                  <Grid item xs={8}>
                    <Typography variant={mediaQuery ? "h6" : "h5"} component="div">
                      {o.name.split("\n")[0]}
                    </Typography>
                  </Grid>
                  <Grid item xs={4}>
                    <Typography variant="caption">Added: {new Date(o.CreationDateUtc).toLocaleDateString()}</Typography>
                    <Typography sx={{ mb: 1.5 }} color="text.secondary">
                      <Grid container item spacing={4}>
                        <Grid item xs={3} md={1} style={{ color: theme.palette.text.primary }}>
                          {isOfferValid(o) ? "✅" : "❌"}
                        </Grid>
                        <Grid item xs={9}>
                          <Typography variant="caption">{o.count} available</Typography>
                        </Grid>
                      </Grid>
                    </Typography>
                  </Grid>
                </Grid>
              </CardContent>
              <CardActions>
                <Button color="secondary" size="large" onClick={() => onSelection(o)}>
                  Select
                </Button>
              </CardActions>
            </Card>
          </Grid>
        ))}
      </Grid>
    </>
  );
};

export default DealSelector;
