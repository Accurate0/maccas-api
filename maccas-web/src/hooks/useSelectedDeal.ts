import { atom, useRecoilState } from "recoil";
import SessionStorageEffect from "../lib/SessionStorageEffect";
import { Offer } from "../types";

const SelectedOffer = atom<Offer | null>({
  key: "selectedOffer",
  default: null,
  effects: [SessionStorageEffect("selectedOffer")],
});

const useSelectedDeal = () => useRecoilState(SelectedOffer);

export default useSelectedDeal;
