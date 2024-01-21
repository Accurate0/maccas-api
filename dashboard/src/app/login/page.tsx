"use client";

import { Callout, Card, TextInput, Title } from "@tremor/react";
import { useFormState } from "react-dom";
import { login } from "../action";
import { SubmitButton } from "@/components/submit-button";
import { ExclamationIcon } from "@heroicons/react/solid";

const initialState = {
  error: "",
};

const Page = () => {
  const [state, formAction] = useFormState(login, initialState);

  return (
    <div className="flex h-screen items-center justify-center">
      <Card className="w-fit">
        <Title>Sign in</Title>
        <div className="pt-4">
          <form action={formAction}>
            <TextInput
              className="mt-4"
              name="username"
              id="username"
              placeholder="Username"
            />
            <TextInput
              className="mt-4"
              type="password"
              name="password"
              id="password"
              placeholder="Password"
            />
            {state.error && (
              <Callout
                className="mt-4"
                title="Error"
                icon={ExclamationIcon}
                color="rose"
              >
                {state.error}
              </Callout>
            )}
            <div className="mt-4">
              <SubmitButton>Login</SubmitButton>
            </div>
          </form>
        </div>
      </Card>
    </div>
  );
};

export default Page;
