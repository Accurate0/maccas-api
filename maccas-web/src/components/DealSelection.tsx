import { Box, Button, Card, CardActions, CardContent, CardMedia, Grid, Typography } from "@mui/material";
import { Container } from "@mui/system";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { IMAGE_BUCKET_BASE } from "../config/api";
import useCode from "../hooks/useCode";
import { Offer } from "../types";

export interface DealSelectionProps {
  selected?: Offer;
}

const DealSelection: React.FC<DealSelectionProps> = ({ selected }) => {
  const navigate = useNavigate();
  const { code, setDeal, remove } = useCode();

  useEffect(() => {
    setDeal(selected);
  }, [selected, setDeal]);

  return (
    <>
      {code && (
        <Container>
          <Grid
            container
            spacing={0}
            direction="column"
            alignItems="center"
            justifyContent="center"
            style={{ minHeight: "100vh" }}
          >
            <Grid item xs={12}>
              <Card variant="outlined">
                <CardMedia component="img" image={`${IMAGE_BUCKET_BASE}/${selected?.imageBaseName}`} />
                <CardContent style={{ margin: "25px 25px 25px 25px" }}>
                  <Typography sx={{ fontSize: 24 }} color="text.primary" gutterBottom>
                    Offer
                  </Typography>
                  <Typography variant="h5" component="div"></Typography>
                  <Typography sx={{ mb: 1.5 }} color="text.secondary">
                    {selected?.name.split("\n")[0]}
                  </Typography>
                  <Typography gutterBottom>{code?.status.message}</Typography>
                  <Typography component="div" gutterBottom>
                    <Box sx={{ fontFamily: "Monospace", fontSize: "h6.fontSize" }}>{code.response?.randomCode}</Box>
                  </Typography>
                </CardContent>
                <CardActions style={{ margin: "25px 25px 25px 25px" }}>
                  <Button
                    color="error"
                    variant="outlined"
                    onClick={async () => {
                      await remove();
                      navigate("/");
                    }}
                  >
                    Remove
                  </Button>
                </CardActions>
              </Card>
            </Grid>
          </Grid>
        </Container>
      )}
    </>
  );
};

export default DealSelection;
