import { getAuth0AvatarSrc } from '@/app/helpers/links';
import { DashboardHeader } from '@/dashboard/components/DashboardHeader';
import { Empty } from '@/dashboard/components/Empty';
import { FilesList } from '@/dashboard/components/FilesList';
import { FilesListEmptyState } from '@/dashboard/components/FilesListEmptyState';
import NewFileButton from '@/dashboard/components/NewFileButton';
import { OnboardingBanner } from '@/dashboard/components/OnboardingBanner';
import { useDashboardRouteLoaderData } from '@/routes/_dashboard';
import { ROUTES } from '@/shared/constants/routes';
import { Add } from '@mui/icons-material';
import { Avatar, AvatarGroup } from '@mui/material';
import { FileIcon } from '@radix-ui/react-icons';
import { Link } from 'react-router-dom';

export const Component = () => {
  const {
    activeTeam: {
      team: { uuid: teamUuid },
      files,
      userMakingRequest: { teamPermissions },
      users,
    },
  } = useDashboardRouteLoaderData();
  const canEdit = teamPermissions.includes('TEAM_EDIT');
  const avatarSxProps = { width: 24, height: 24, fontSize: '.875rem' };

  const usersById: Record<
    number,
    { name: string | undefined; picture: string | undefined; email: string | undefined }
  > = users.reduce(
    (acc, user) => ({
      ...acc,
      [user.id]: { name: user.name, picture: user.picture, email: user.email },
    }),
    {}
  );

  return (
    <div className="flex flex-grow flex-col">
      <OnboardingBanner />

      <DashboardHeader
        title="Team files"
        actions={
          <div className={`flex items-center gap-2`}>
            <div className="hidden lg:block">
              <Link to={ROUTES.TEAM_MEMBERS(teamUuid)}>
                {/* TODO(ayush): create custom AvatarGroup component */}
                <AvatarGroup
                  max={6}
                  sx={{ cursor: 'pointer', pr: 0 }}
                  slotProps={{ additionalAvatar: { sx: avatarSxProps } }}
                >
                  <Avatar alt="Add" sx={{ ...avatarSxProps }} className="text-sm">
                    <Add fontSize="inherit" />
                  </Avatar>
                  {users.map((user, key) => (
                    <Avatar
                      key={key}
                      alt={user.name}
                      src={getAuth0AvatarSrc(user.picture)}
                      sx={avatarSxProps}
                      imgProps={{ crossOrigin: 'anonymous' }}
                    />
                  ))}
                </AvatarGroup>
              </Link>
            </div>
            {canEdit && <NewFileButton isPrivate={false} />}
          </div>
        }
      />

      <FilesList
        files={files.map(({ file, userMakingRequest }) => {
          const creator = usersById[file.creatorId];
          return {
            name: file.name,
            createdDate: file.createdDate,
            updatedDate: file.updatedDate,
            thumbnail: file.thumbnail,
            uuid: file.uuid,
            publicLinkAccess: file.publicLinkAccess,
            permissions: userMakingRequest.filePermissions,
            creator,
          };
        })}
        teamUuid={teamUuid}
        isPrivate={false}
        emptyState={
          canEdit ? (
            <FilesListEmptyState />
          ) : (
            <Empty
              title="No team files yet"
              description={`Files created by your team members will show up here.`}
              Icon={FileIcon}
            />
          )
        }
      />
    </div>
  );
};
