import Form from "@/components/form";
import { auth } from "@/lucia";
import { Button, Card, TextInput, Title } from "@tremor/react";
import * as context from "next/headers";
import { redirect } from "next/navigation";

const Page = async () => {
  const authRequest = auth.handleRequest("GET", context);
  const session = await authRequest.validate();
  if (session) redirect("/");

  return (
    <div className="flex h-screen items-center justify-center">
      <Card className="w-fit">
        <Title>Sign in</Title>
        <div className="pt-4">
          <Form action="/api/login">
            <TextInput name="username" id="username" placeholder="Username" />
            <br />
            <TextInput
              type="password"
              name="password"
              id="password"
              placeholder="Password"
            />
            <br />
            <Button type="submit">Login</Button>
          </Form>
        </div>
      </Card>
    </div>
  );
};

export default Page;
