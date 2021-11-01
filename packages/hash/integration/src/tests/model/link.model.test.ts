import "../loadTestEnv";
import { PostgresAdapter } from "@hashintel/hash-backend/src/db";
import {
  Entity,
  EntityType,
  Link,
  User,
} from "@hashintel/hash-backend/src/model";
import { WayToUseHash } from "@hashintel/hash-backend/src/graphql/apiTypes.gen";
import { IntegrationTestsHandler } from "../setup";

let handler: IntegrationTestsHandler;

let db: PostgresAdapter;

let existingUser: User;

let dummyEntityType: EntityType;

beforeAll(async () => {
  handler = new IntegrationTestsHandler();
  await handler.init();

  db = new PostgresAdapter({
    host: "localhost",
    user: "postgres",
    port: 5432,
    database: "integration_tests",
    password: "postgres",
  });

  existingUser = await User.createUser(db)({
    shortname: "test-user",
    preferredName: "Alice",
    emails: [{ address: "alice@hash.test", primary: true, verified: true }],
    memberOf: [],
    infoProvidedAtSignup: { usingHow: WayToUseHash.ByThemselves },
  });

  dummyEntityType = await EntityType.create(db)({
    accountId: existingUser.accountId,
    createdById: existingUser.entityId,
    name: "Dummy",
  });
});

describe("Link model class ", () => {
  it("static isPathValid method correctly validates JSON path correctly", () => {
    expect(Link.isPathValid("$.this[0].path['should'].be[\"valid\"]")).toBe(
      true
    );
    expect(Link.isPathValid("thispathisn'tvalid")).toBe(false);
    expect(Link.isPathValid("$.this.is.not.valid.")).toBe(false);
  });

  it("static create method can create a link", async () => {
    const accountId = existingUser.accountId;
    const createdById = existingUser.entityId;

    const entity1 = await Entity.create(db)({
      accountId,
      createdById,
      versioned: true,
      entityTypeId: dummyEntityType.entityId,
      properties: {},
    });

    const entity2 = await Entity.create(db)({
      accountId,
      createdById,
      versioned: true,
      entityTypeId: dummyEntityType.entityId,
      properties: {},
    });

    const link = await Link.create(db)({
      path: "$.linkName",
      source: entity1,
      destination: entity2,
    });

    expect(link.srcEntityId).toBe(entity1.entityId);
    expect(link.dstEntityId).toBe(entity2.entityId);
  });

  it("static get method can retrieve a link from the datastore", async () => {
    const accountId = existingUser.accountId;
    const createdById = existingUser.entityId;

    const entity1 = await Entity.create(db)({
      accountId,
      createdById,
      versioned: true,
      entityTypeId: dummyEntityType.entityId,
      properties: {},
    });

    const entity2 = await Entity.create(db)({
      accountId,
      createdById,
      versioned: true,
      entityTypeId: dummyEntityType.entityId,
      properties: {},
    });

    const link = await Link.create(db)({
      path: "$.linkName",
      source: entity1,
      destination: entity2,
    });

    const retrievedLink = (await Link.get(db)({
      accountId,
      linkId: link.linkId,
    }))!;

    expect(retrievedLink).not.toBeNull();
    expect(retrievedLink.linkId).toBe(link.linkId);
    expect(retrievedLink.createdAt).toEqual(link.createdAt);
    expect(retrievedLink.srcAccountId).toBe(link.srcAccountId);
    expect(retrievedLink.srcEntityId).toBe(link.srcEntityId);
    expect(retrievedLink.srcEntityVersionIds).toEqual(link.srcEntityVersionIds);
    expect(retrievedLink.dstAccountId).toBe(link.dstAccountId);
    expect(retrievedLink.dstEntityId).toBe(link.dstEntityId);
    expect(retrievedLink.dstEntityVersionId).toBe(link.dstEntityVersionId);
  });

  it("can create outgoing link on non-versioned entity", async () => {
    const accountId = existingUser.accountId;
    const createdById = existingUser.entityId;

    const [sourceEntity, destinationEntity] = await Promise.all([
      Entity.create(db)({
        accountId,
        createdById,
        versioned: false,
        entityTypeId: dummyEntityType.entityId,
        properties: {},
      }),
      Entity.create(db)({
        accountId,
        createdById,
        versioned: false,
        entityTypeId: dummyEntityType.entityId,
        properties: {},
      }),
    ]);

    const intialSourceEntityId = sourceEntity.entityVersionId;

    const link = await Link.create(db)({
      path: "test",
      source: sourceEntity,
      destination: destinationEntity,
    });

    expect(link.srcEntityId).toBe(sourceEntity.entityId);
    expect(Array.from(link.srcEntityVersionIds)).toEqual([intialSourceEntityId]);
    expect(sourceEntity.entityVersionId).toBe(intialSourceEntityId);
  });

  it("can create outgoing link on versioned source entity", async () => {
    const accountId = existingUser.accountId;
    const createdById = existingUser.entityId;

    const [versionedSourceEntity, destinationEntity] = await Promise.all([
      Entity.create(db)({
        accountId,
        createdById,
        versioned: true,
        entityTypeId: dummyEntityType.entityId,
        properties: {},
      }),
      Entity.create(db)({
        accountId,
        createdById,
        versioned: false,
        entityTypeId: dummyEntityType.entityId,
        properties: {},
      }),
    ]);

    const entityVersionIds = [versionedSourceEntity.entityVersionId];

    const link = await Link.create(db)({
      path: "test",
      source: versionedSourceEntity,
      destination: destinationEntity,
    });

    entityVersionIds.push(versionedSourceEntity.entityVersionId);

    expect(entityVersionIds[0]).not.toBe(entityVersionIds[1]);

    // The first version of the entity should have 0 outgoing links

    const e1 = (await Entity.getEntity(db)({
      accountId,
      entityVersionId: entityVersionIds[0],
    }))!;

    expect(e1).not.toBeNull();

    const e1OutgoingLinks = await e1.getOutgoingLinks(db);

    expect(e1OutgoingLinks.length).toBe(0);

    // The second version of the entity should have 1 outgoing link

    const e2 = (await Entity.getEntity(db)({
      accountId,
      entityVersionId: entityVersionIds[1],
    }))!;

    expect(e2).not.toBeNull();

    const e2OutgoingLinks = await e2.getOutgoingLinks(db);

    expect(e2OutgoingLinks.length).toBe(1);
    expect(e2OutgoingLinks[0].linkId).toBe(link.linkId);
  });
});

afterAll(async () => {
  await handler.close();
  await db.close();
});
