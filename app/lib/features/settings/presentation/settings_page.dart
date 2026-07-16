import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../auth/application/auth_provider.dart';
import '../../albums/data/album_repository.dart';
import '../data/api_key_repository.dart';
import '../../tags/data/tag_repository.dart';
import '../data/partner_repository.dart';
import '../data/shared_link_repository.dart';
import '../data/system_config_repository.dart';
import '../data/user_admin_repository.dart';
import 'immich_derivative_panel.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final auth = ref.watch(authStateProvider);
    final apiKeys = ref.watch(apiKeysProvider);
    final tags = ref.watch(tagsProvider);
    final partners = ref.watch(partnersProvider);
    final sharedLinks = ref.watch(sharedLinksProvider);
    final storageTemplate = ref.watch(storageTemplateProvider);
    final oauthConfig = ref.watch(oauthConfigProvider);
    final adminUsers = auth.user?.isAdmin == true
        ? ref.watch(adminUsersProvider)
        : const AsyncValue.data(null);

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
            leading: const Icon(Icons.folder_copy_outlined),
            title: const Text('Storage template'),
            subtitle: storageTemplate.when(
              loading: () => const Text('Loading'),
              error: (error, _) => Text('$error'),
              data: (config) =>
                  Text(config.enabled ? config.template : 'Disabled'),
            ),
            trailing: IconButton(
              icon: const Icon(Icons.edit_outlined),
              onPressed: () => _editStorageTemplate(context, ref),
            ),
          ),
          ListTile(
            leading: const Icon(Icons.login_outlined),
            title: const Text('OAuth'),
            subtitle: oauthConfig.when(
              loading: () => const Text('Loading'),
              error: (error, _) => Text('$error'),
              data: (config) =>
                  Text(config.enabled ? config.clientId : 'Disabled'),
            ),
            trailing: IconButton(
              icon: const Icon(Icons.edit_outlined),
              onPressed: () => _editOAuth(context, ref),
            ),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.key_outlined),
            title: const Text('API keys'),
            trailing: IconButton(
              icon: const Icon(Icons.add),
              onPressed: () => _createApiKey(context, ref),
            ),
          ),
          apiKeys.when(
            loading: () => const ListTile(title: LinearProgressIndicator()),
            error: (error, _) => ListTile(title: Text('$error')),
            data: (keys) => Column(
              children: [
                for (final key in keys)
                  ListTile(
                    dense: true,
                    title: Text(key.name),
                    subtitle: Text(key.id),
                    trailing: IconButton(
                      icon: const Icon(Icons.delete_outline),
                      onPressed: () async {
                        await ref.read(apiKeyRepositoryProvider).delete(key.id);
                        ref.invalidate(apiKeysProvider);
                      },
                    ),
                  ),
              ],
            ),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.sell_outlined),
            title: const Text('Tags'),
            trailing: IconButton(
              icon: const Icon(Icons.add),
              onPressed: () => _createTag(context, ref),
            ),
          ),
          tags.when(
            loading: () => const ListTile(title: LinearProgressIndicator()),
            error: (error, _) => ListTile(title: Text('$error')),
            data: (tags) => Column(
              children: [
                for (final tag in tags)
                  ListTile(
                    dense: true,
                    title: Text(tag.name),
                    subtitle: Text(tag.value),
                  ),
              ],
            ),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.people_alt_outlined),
            title: const Text('Partner sharing'),
            trailing: IconButton(
              icon: const Icon(Icons.person_add_alt_outlined),
              onPressed: () => _createPartner(context, ref),
            ),
          ),
          partners.when(
            loading: () => const ListTile(title: LinearProgressIndicator()),
            error: (error, _) => ListTile(title: Text('$error')),
            data: (partners) => Column(
              children: [
                for (final partner in partners)
                  ListTile(
                    dense: true,
                    title: Text(
                      partner.name.isEmpty ? partner.email : partner.name,
                    ),
                    subtitle: Text(
                      partner.inTimeline
                          ? 'Visible in timeline'
                          : 'Hidden from timeline',
                    ),
                    trailing: IconButton(
                      icon: const Icon(Icons.delete_outline),
                      onPressed: () async {
                        await ref
                            .read(partnerRepositoryProvider)
                            .remove(partner.id);
                        ref.invalidate(partnersProvider);
                      },
                    ),
                  ),
              ],
            ),
          ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.link_outlined),
            title: const Text('Shared links'),
          ),
          sharedLinks.when(
            loading: () => const ListTile(title: LinearProgressIndicator()),
            error: (error, _) => ListTile(title: Text('$error')),
            data: (links) => Column(
              children: [
                for (final link in links)
                  ListTile(
                    dense: true,
                    title: SelectableText(link.key),
                    subtitle: Text('${link.assets.length} asset(s)'),
                    trailing: IconButton(
                      icon: const Icon(Icons.delete_outline),
                      onPressed: () async {
                        await ref
                            .read(sharedLinkRepositoryProvider)
                            .delete(link.id);
                        ref.invalidate(sharedLinksProvider);
                      },
                    ),
                  ),
              ],
            ),
          ),
          if (auth.user?.isAdmin == true) ...[
            const Divider(),
            const ImmichDerivativePanel(),
            const Divider(),
            ListTile(
              leading: const Icon(Icons.admin_panel_settings_outlined),
              title: const Text('Users'),
              trailing: IconButton(
                icon: const Icon(Icons.person_add_alt_outlined),
                onPressed: () => _createUser(context, ref),
              ),
            ),
            adminUsers.when(
              loading: () => const ListTile(title: LinearProgressIndicator()),
              error: (error, _) => ListTile(title: Text('$error')),
              data: (users) => Column(
                children: [
                  for (final user in users ?? const [])
                    ListTile(
                      dense: true,
                      title: Text(user.name),
                      subtitle: Text(user.email),
                      trailing: user.isAdmin
                          ? const Icon(Icons.verified_user_outlined)
                          : null,
                    ),
                ],
              ),
            ),
          ],
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

  Future<void> _createApiKey(BuildContext context, WidgetRef ref) async {
    final controller = TextEditingController(text: 'Domus API Key');
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('New API key'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(labelText: 'Name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('Create'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (name == null || name.trim().isEmpty) return;
    final key = await ref.read(apiKeyRepositoryProvider).create(name.trim());
    ref.invalidate(apiKeysProvider);
    if (!context.mounted || key.secret == null) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('API key created'),
        content: SelectableText(key.secret!),
        actions: [
          FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Done'),
          ),
        ],
      ),
    );
  }

  Future<void> _editStorageTemplate(BuildContext context, WidgetRef ref) async {
    final current = await ref
        .read(systemConfigRepositoryProvider)
        .getStorageTemplate();
    if (!context.mounted) return;
    var enabled = current.enabled;
    final controller = TextEditingController(text: current.template);
    final save = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Storage template'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              SwitchListTile(
                value: enabled,
                onChanged: (value) => setState(() => enabled = value),
                title: const Text('Enabled'),
              ),
              TextField(
                controller: controller,
                decoration: const InputDecoration(labelText: 'Template'),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: const Text('Save'),
            ),
          ],
        ),
      ),
    );
    final template = controller.text.trim();
    controller.dispose();
    if (save != true || template.isEmpty) return;
    await ref
        .read(systemConfigRepositoryProvider)
        .setStorageTemplate(enabled: enabled, template: template);
    ref.invalidate(storageTemplateProvider);
  }

  Future<void> _editOAuth(BuildContext context, WidgetRef ref) async {
    final current = await ref.read(systemConfigRepositoryProvider).getOAuth();
    if (!context.mounted) return;
    var enabled = current.enabled;
    final authorizeUrl = TextEditingController(text: current.authorizeUrl);
    final tokenEndpoint = TextEditingController(text: current.tokenEndpoint);
    final userinfoEndpoint = TextEditingController(
      text: current.userinfoEndpoint,
    );
    final clientId = TextEditingController(text: current.clientId);
    final clientSecret = TextEditingController(text: current.clientSecret);
    final scope = TextEditingController(text: current.scope);
    final save = await showDialog<bool>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('OAuth'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                SwitchListTile(
                  value: enabled,
                  onChanged: (value) => setState(() => enabled = value),
                  title: const Text('Enabled'),
                ),
                TextField(
                  controller: authorizeUrl,
                  decoration: const InputDecoration(labelText: 'Authorize URL'),
                ),
                TextField(
                  controller: tokenEndpoint,
                  decoration: const InputDecoration(
                    labelText: 'Token endpoint',
                  ),
                ),
                TextField(
                  controller: userinfoEndpoint,
                  decoration: const InputDecoration(
                    labelText: 'Userinfo endpoint',
                  ),
                ),
                TextField(
                  controller: clientId,
                  decoration: const InputDecoration(labelText: 'Client ID'),
                ),
                TextField(
                  controller: clientSecret,
                  decoration: const InputDecoration(labelText: 'Client secret'),
                ),
                TextField(
                  controller: scope,
                  decoration: const InputDecoration(labelText: 'Scope'),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: const Text('Save'),
            ),
          ],
        ),
      ),
    );
    final config = OAuthConfig(
      enabled: enabled,
      authorizeUrl: authorizeUrl.text.trim(),
      tokenEndpoint: tokenEndpoint.text.trim(),
      userinfoEndpoint: userinfoEndpoint.text.trim(),
      clientId: clientId.text.trim(),
      clientSecret: clientSecret.text,
      scope: scope.text.trim().isEmpty
          ? 'openid email profile'
          : scope.text.trim(),
    );
    for (final controller in [
      authorizeUrl,
      tokenEndpoint,
      userinfoEndpoint,
      clientId,
      clientSecret,
      scope,
    ]) {
      controller.dispose();
    }
    if (save != true) return;
    await ref.read(systemConfigRepositoryProvider).setOAuth(config);
    ref.invalidate(oauthConfigProvider);
  }

  Future<void> _createTag(BuildContext context, WidgetRef ref) async {
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('New tag'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(labelText: 'Name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('Create'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (name == null || name.trim().isEmpty) return;
    await ref.read(tagRepositoryProvider).create(name.trim());
    ref.invalidate(tagsProvider);
  }

  Future<void> _createPartner(BuildContext context, WidgetRef ref) async {
    final users = await ref.read(albumRepositoryProvider).listUsers();
    if (!context.mounted) return;
    final userId = await showDialog<String>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Add partner'),
        children: [
          for (final user in users)
            SimpleDialogOption(
              onPressed: () => Navigator.of(context).pop(user.id),
              child: ListTile(
                leading: const Icon(Icons.person_outline),
                title: Text(user.name),
                subtitle: Text(user.email),
              ),
            ),
        ],
      ),
    );
    if (userId == null) return;
    await ref.read(partnerRepositoryProvider).create(userId);
    ref.invalidate(partnersProvider);
  }

  Future<void> _createUser(BuildContext context, WidgetRef ref) async {
    final email = TextEditingController();
    final name = TextEditingController();
    final password = TextEditingController();
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('New user'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: email,
              autofocus: true,
              decoration: const InputDecoration(labelText: 'Email'),
            ),
            TextField(
              controller: name,
              decoration: const InputDecoration(labelText: 'Name'),
            ),
            TextField(
              controller: password,
              obscureText: true,
              decoration: const InputDecoration(labelText: 'Password'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Create'),
          ),
        ],
      ),
    );
    final emailValue = email.text.trim();
    final nameValue = name.text.trim();
    final passwordValue = password.text;
    email.dispose();
    name.dispose();
    password.dispose();
    if (result != true ||
        emailValue.isEmpty ||
        nameValue.isEmpty ||
        passwordValue.isEmpty) {
      return;
    }
    await ref
        .read(userAdminRepositoryProvider)
        .create(email: emailValue, password: passwordValue, name: nameValue);
    ref.invalidate(adminUsersProvider);
  }
}
