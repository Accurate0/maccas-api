import { Box, Button, Card, CardActions, CardContent, CardMedia, Grid, Typography } from "@mui/material";
import { Container } from "@mui/system";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { IMAGE_BUCKET_BASE } from "../config/api";
import useCode from "../hooks/useCode";
import useSelectedDeal from "../hooks/useSelectedDeal";

export interface DealSelectionProps {}

const DealSelection: React.FC<DealSelectionProps> = () => {
  const [selectedDeal] = useSelectedDeal();
  const navigate = useNavigate();
  const { code, setDeal, remove, refreshCode } = useCode();

  useEffect(() => {
    setDeal(selectedDeal);
  }, [selectedDeal, setDeal]);

  return (
    <>
      {code && selectedDeal && (
        <Container>
          <Grid
            container
            spacing={0}
            direction="column"
            alignItems="center"
            justifyContent="center"
            style={{ minHeight: "100vh" }}
            paddingTop={8}
            paddingBottom={4}
          >
            <Grid item xs={12}>
              <Card variant="outlined">
                <CardMedia
                  height="380"
                  width="380"
                  component="img"
                  image={`${IMAGE_BUCKET_BASE}/${selectedDeal?.imageBaseName}`}
                />
                <CardContent style={{ margin: "25px 25px 25px 25px" }}>
                  <Typography sx={{ fontSize: 24 }} color="text.primary" gutterBottom>
                    Offer
                  </Typography>
                  <Typography variant="h5" component="div"></Typography>
                  <Typography sx={{ mb: 1.5 }} color="text.secondary">
                    {selectedDeal?.shortName}
                  </Typography>
                  <Typography gutterBottom>{code?.status.message}</Typography>
                  <Typography component="div" gutterBottom>
                    <Box sx={{ fontFamily: "Monospace", fontSize: "h6.fontSize" }}>{code.response?.randomCode}</Box>
                  </Typography>
                </CardContent>
                <CardActions style={{ margin: "25px 25px 25px 25px" }}>
                  <Grid container justifyContent="space-between">
                    <Grid item>
                      <Button
                        color="success"
                        variant="contained"
                        onClick={async () => {
                          await refreshCode();
                        }}
                      >
                        Refresh
                      </Button>
                    </Grid>
                    <Grid item>
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
                    </Grid>
                  </Grid>
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
