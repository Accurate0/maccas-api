import {
  GetOffersDocument,
  GetOffersQuery,
} from "@/graphql/__generated__/graphql";
import { getClient } from "@/lib/apollo";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import Image from "next/image";
import { Badge } from "@/components/ui/badge";
import { userSession } from "@/lib/session";

export default async function Offers() {
  await userSession();

  const { data } = await getClient().query<GetOffersQuery>({
    query: GetOffersDocument,
  });

  return (
    <div className="grid grid-flow-row gap-4">
      {data.offers.map(({ shortName, name, count, id, imageUrl }) => (
        <Card key={id}>
          <div className="grid grid-flow-col justify-between">
            <CardHeader className="grid justify-between">
              <CardTitle>{shortName}</CardTitle>
              <CardDescription>{name}</CardDescription>
              <Badge className="w-fit h-fit">{count} available</Badge>
            </CardHeader>
            <CardHeader>
              <Image src={imageUrl} alt={shortName} width={100} height={100} />
            </CardHeader>
          </div>
          {/* <CardFooter>
            <Skeleton className="h-2 w-full bg-primary/50" />
          </CardFooter> */}
        </Card>
      ))}
    </div>
  );
}
