import request from 'supertest';
import { app } from '../../app';
import dbClient from '../../dbClient';
import { createTeam, createUser } from '../../tests/testDataGenerator';

beforeAll(async () => {
  const teamOwner = await createUser({ auth0Id: 'teamOwner' });
  await createUser({ auth0Id: 'teamViewer' });

  await createTeam({
    team: {
      uuid: '00000000-0000-0000-0000-000000000000',
    },
    users: [{ userId: teamOwner.id, role: 'OWNER' }],
    connections: [{ uuid: '10000000-0000-0000-0000-000000000000', type: 'POSTGRES' }],
  });
});

afterAll(async () => {
  await dbClient.$transaction([
    dbClient.connection.deleteMany(),
    dbClient.userTeamRole.deleteMany(),
    dbClient.team.deleteMany(),
    dbClient.user.deleteMany(),
  ]);
});

describe('DELETE /v0/connections/:uuid', () => {
  describe('a team viewer', () => {
    it('responds with a 403', async () => {
      await request(app)
        .get('/v0/connections/10000000-0000-0000-0000-000000000000')
        .set('Authorization', `Bearer ValidToken teamViewer`)
        .expect(403);
    });
  });

  describe('a team editor/owner', () => {
    it('deletes the connection', async () => {
      await request(app)
        .delete('/v0/connections/10000000-0000-0000-0000-000000000000')
        .set('Authorization', `Bearer ValidToken teamOwner`)
        .expect(200)
        .expect(({ body }) => {
          expect(body.message).toBeDefined();
        });
    });
  });
});