import {
  Grid,
  Button,
  CardActions,
  CardContent,
  Typography,
  useMediaQuery,
  CardMedia,
  Card,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
} from "@mui/material";
import moment from "moment";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { IMAGE_BUCKET_BASE } from "../config/api";
import useDeals from "../hooks/useDeals";
import useLastRefresh from "../hooks/useLastRefresh";
import useSelectedDeal from "../hooks/useSelectedDeal";
import { theme } from "../styles";
import { Offer } from "../types";

export interface DealSelectorProps {}

const DealSelector: React.FC<DealSelectorProps> = () => {
  const navigate = useNavigate();
  const [, setSelectedDeal] = useSelectedDeal();
  const deals = useDeals();
  const mediaQuery = useMediaQuery(theme.breakpoints.down("md"));
  const [open, setOpen] = useState(false);
  const [dialogFor, setDialogFor] = useState<Offer>();
  const handleClickOpen = () => setOpen(true);
  const handleClose = () => setOpen(false);
  useLastRefresh();

  const isOfferValid = (deal: Offer) => {
    const from = moment.utc(deal.validFromUTC);
    const to = moment.utc(deal.validToUTC);
    const now = new Date();

    return moment.utc(now).isBetween(from, to);
  };

  return (
    <>
      <Dialog open={open} onClose={handleClose}>
        <DialogTitle>{dialogFor?.name.split("\n")[0]}</DialogTitle>
        <DialogContent>
          <Grid container spacing={2}>
            <Grid item>
              <DialogContentText>
                Valid From: {moment.utc(dialogFor?.validFromUTC).local().format("LLL")}
              </DialogContentText>
              <DialogContentText>Valid To: {moment.utc(dialogFor?.validToUTC).local().format("LLL")}</DialogContentText>
            </Grid>
            <Grid item>
              <DialogContentText>{dialogFor?.description}</DialogContentText>
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button color="secondary" onClick={handleClose}>
            Close
          </Button>
        </DialogActions>
      </Dialog>
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
                <Grid container justifyContent="space-between">
                  <Grid item>
                    <Button
                      color="secondary"
                      size="large"
                      onClick={() => {
                        setSelectedDeal(o);
                        navigate("/code");
                      }}
                    >
                      Select
                    </Button>
                  </Grid>
                  <Grid item>
                    <Button
                      color="secondary"
                      size="large"
                      onClick={() => {
                        setDialogFor(o);
                        handleClickOpen();
                      }}
                    >
                      Details
                    </Button>
                  </Grid>
                </Grid>
              </CardActions>
            </Card>
          </Grid>
        ))}
      </Grid>
    </>
  );
};

export default DealSelector;
