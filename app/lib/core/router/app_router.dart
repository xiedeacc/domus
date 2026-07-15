import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../features/albums/presentation/album_detail_page.dart';
import '../../features/albums/presentation/albums_page.dart';
import '../../features/asset_viewer/presentation/asset_viewer_page.dart';
import '../../features/auth/application/auth_provider.dart';
import '../../features/auth/presentation/login_page.dart';
import '../../features/backup/presentation/backup_page.dart';
import '../../features/folders/presentation/folders_page.dart';
import '../../features/library/presentation/library_page.dart';
import '../../features/search/presentation/search_page.dart';
import '../../features/search/presentation/global_map_page.dart';
import '../../features/settings/presentation/settings_page.dart';
import '../../features/shell/home_shell.dart';
import '../../features/timeline/presentation/timeline_page.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  final auth = ref.watch(authStateProvider);

  return GoRouter(
    initialLocation: '/timeline',
    redirect: (context, state) {
      final loggingIn = state.matchedLocation == '/login';
      if (!auth.isAuthenticated && !loggingIn) return '/login';
      if (auth.isAuthenticated && loggingIn) return '/timeline';
      return null;
    },
    routes: [
      GoRoute(path: '/login', builder: (_, _) => const LoginPage()),
      // Main shell: bottom navigation (mobile) / side rail (web & tablet).
      StatefulShellRoute.indexedStack(
        builder: (_, _, shell) => HomeShell(shell: shell),
        branches: [
          StatefulShellBranch(
            routes: [
              GoRoute(
                path: '/timeline',
                builder: (_, _) => const TimelinePage(),
              ),
            ],
          ),
          StatefulShellBranch(
            routes: [
              GoRoute(path: '/search', builder: (_, _) => const SearchPage()),
            ],
          ),
          StatefulShellBranch(
            routes: [
              GoRoute(
                path: '/albums',
                builder: (_, _) => const AlbumsPage(),
                routes: [
                  GoRoute(
                    path: ':id',
                    builder: (_, state) =>
                        AlbumDetailPage(albumId: state.pathParameters['id']!),
                  ),
                ],
              ),
            ],
          ),
          StatefulShellBranch(
            routes: [
              GoRoute(path: '/library', builder: (_, _) => const LibraryPage()),
            ],
          ),
        ],
      ),
      GoRoute(
        path: '/asset/:id',
        builder: (_, state) =>
            AssetViewerPage(assetId: state.pathParameters['id']!),
      ),
      GoRoute(path: '/backup', builder: (_, _) => const BackupPage()),
      GoRoute(path: '/folders', builder: (_, _) => const FoldersPage()),
      GoRoute(path: '/map', builder: (_, _) => const GlobalMapPage()),
      GoRoute(
        path: '/folders/detail',
        builder: (_, state) => FolderDetailPage(path: state.extra! as String),
      ),
      GoRoute(path: '/settings', builder: (_, _) => const SettingsPage()),
    ],
  );
});
