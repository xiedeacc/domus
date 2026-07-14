import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../auth/application/auth_provider.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final auth = ref.watch(authStateProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: [
          if (auth.user != null)
            ListTile(
              leading: const CircleAvatar(child: Icon(Icons.person)),
              title: Text(auth.user!.name),
              subtitle: Text(auth.user!.email),
            ),
          const Divider(),
          const ListTile(
            leading: Icon(Icons.photo_size_select_actual_outlined),
            title: Text('Image quality'),
            subtitle: Text('Preview quality for the timeline'),
          ),
          const ListTile(
            leading: Icon(Icons.language_outlined),
            title: Text('Language'),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.logout),
            title: const Text('Log out'),
            onTap: () => ref.read(authStateProvider.notifier).logout(),
          ),
        ],
      ),
    );
  }
}
