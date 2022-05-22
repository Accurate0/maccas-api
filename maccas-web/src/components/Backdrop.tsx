import { CircularProgress, Backdrop as MuiBackdrop } from "@mui/material";
import { atom, selector, useRecoilState } from "recoil";

const BackdropState = atom<number>({
  key: "BackdropState",
  default: 0,
});

export const BackdropSelector = selector<boolean>({
  key: "BackdropSelector",
  get: ({ get }) => get(BackdropState) > 0,
  set: ({ set, get }, newValue) => {
    const oldValue = get(BackdropState);
    newValue ? set(BackdropState, oldValue + 1) : set(BackdropState, oldValue - 1 < 0 ? 0 : oldValue - 1);
  },
});

const Backdrop = () => {
  const [state] = useRecoilState(BackdropSelector);

  return (
    <MuiBackdrop sx={{ color: "#fff", zIndex: (theme) => theme.zIndex.drawer + 1 }} open={state}>
      <CircularProgress color="inherit" />
    </MuiBackdrop>
  );
};

export default Backdrop;
