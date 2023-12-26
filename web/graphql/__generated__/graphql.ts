/* eslint-disable */
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /**
   * ISO 8601 combined date and time without timezone.
   *
   * # Examples
   *
   * * `2015-07-01T08:59:60.123`,
   */
  NaiveDateTime: { input: any; output: any; }
  /**
   * A UUID is a unique 128-bit number, stored as 16 octets. UUIDs are parsed as
   * Strings within GraphQL. UUIDs are used to assign unique identifiers to
   * entities without requiring a central allocating authority.
   *
   * # References
   *
   * * [Wikipedia: Universally Unique Identifier](http://en.wikipedia.org/wiki/Universally_unique_identifier)
   * * [RFC4122: A Universally Unique IDentifier (UUID) URN Namespace](http://tools.ietf.org/html/rfc4122)
   */
  UUID: { input: any; output: any; }
};

export type HealthResponse = {
  __typename?: 'HealthResponse';
  databaseHealthy: Scalars['Boolean']['output'];
};

export type Offer = {
  __typename?: 'Offer';
  count: Scalars['Int']['output'];
  creationDate: Scalars['NaiveDateTime']['output'];
  description: Scalars['String']['output'];
  id: Scalars['UUID']['output'];
  imageUrl: Scalars['String']['output'];
  name: Scalars['String']['output'];
  offerId: Scalars['Int']['output'];
  price?: Maybe<Scalars['Float']['output']>;
  shortName: Scalars['String']['output'];
  validFrom: Scalars['NaiveDateTime']['output'];
  validTo: Scalars['NaiveDateTime']['output'];
};

export type Points = {
  __typename?: 'Points';
  accountId: Scalars['UUID']['output'];
  currentPoints: Scalars['Int']['output'];
  lifetimePoints: Scalars['Int']['output'];
};

export type QueryRoot = {
  __typename?: 'QueryRoot';
  health: HealthResponse;
  offers: Array<Offer>;
  points: Array<Points>;
};

export type GetOffersQueryVariables = Exact<{ [key: string]: never; }>;


export type GetOffersQuery = { __typename?: 'QueryRoot', offers: Array<{ __typename?: 'Offer', shortName: string, imageUrl: string, name: string, count: number, id: any }> };


export const GetOffersDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetOffers"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"offers"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shortName"}},{"kind":"Field","name":{"kind":"Name","value":"imageUrl"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"count"}},{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<GetOffersQuery, GetOffersQueryVariables>;