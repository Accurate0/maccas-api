import { useRecoilState } from "recoil";
import { BackdropSelector } from "../components/Backdrop";

const useSetBackdrop = () => {
  const [, setBackdrop] = useRecoilState(BackdropSelector);
  return setBackdrop;
};

export default useSetBackdrop;
