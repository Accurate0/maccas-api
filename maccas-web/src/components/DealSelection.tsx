import { Box, Button, Card, CardContent, Grid, Typography } from "@mui/material";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
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
        <Card variant="outlined">
          <CardContent style={{ margin: "25px 25px 25px 25px" }}>
            <Grid container direction="column" spacing={1}>
              <Grid item>
                <Typography sx={{ fontSize: 24 }} color="text.primary" gutterBottom>
                  Offer
                </Typography>
                <Typography variant="h5" component="div"></Typography>
                <Typography sx={{ mb: 1.5 }} color="text.secondary">
                  {selected?.name.split("\n")[0]}
                </Typography>
                <Typography gutterBottom>{code?.status.message}</Typography>
                <Typography component="div" gutterBottom>
                  <Box sx={{ fontFamily: "Monospace", fontSize: "h6.fontSize" }}>{"2321"}</Box>
                </Typography>
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
          </CardContent>
        </Card>
      )}
    </>
  );
};

export default DealSelection;
