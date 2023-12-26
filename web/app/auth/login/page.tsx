"use client";

import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { LoginState, login } from "@/app/actions/login";
import { useFormState } from "react-dom";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ExclamationTriangleIcon } from "@radix-ui/react-icons";
import SubmitButton from "@/components/submit-button";

export default function Login() {
  const [state, formAction] = useFormState<LoginState, FormData>(login, {
    error: null,
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Login</CardTitle>
      </CardHeader>
      <form action={formAction}>
        <CardContent>
          <div className="grid w-full items-center gap-4">
            <Label htmlFor="username">Username</Label>
            <Input
              id="username"
              type="username"
              placeholder="Username"
              name="username"
            />
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              type="password"
              placeholder="Password"
              name="password"
            />
          </div>
        </CardContent>
        <CardFooter className="w-full">
          <div className="w-full">
            {state.error && (
              <div className="mb-4 w-full">
                <Alert variant="destructive">
                  <ExclamationTriangleIcon className="h-4 w-4" />
                  <AlertTitle>Error</AlertTitle>
                  <AlertDescription>{state.error}</AlertDescription>
                </Alert>
              </div>
            )}
            <SubmitButton>Login</SubmitButton>
          </div>
        </CardFooter>
      </form>
    </Card>
  );
}
