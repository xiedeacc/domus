import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';

import '../application/auth_provider.dart';
import '../data/server_discovery.dart';

class LoginPage extends ConsumerStatefulWidget {
  const LoginPage({super.key});

  @override
  ConsumerState<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends ConsumerState<LoginPage> {
  static const _redirectUri = 'domus://oauth';

  final _serverController = TextEditingController();
  final _emailController = TextEditingController(text: 'xiedeacc@gmail.com');
  final _passwordController = TextEditingController(text: 'qh6288QHW');
  StreamSubscription<Uri>? _linkSub;
  bool _discovering = false;

  @override
  void initState() {
    super.initState();
    _discoverServer();
    _listenForOAuthCallback();
  }

  @override
  void dispose() {
    _linkSub?.cancel();
    _serverController.dispose();
    _emailController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    await ref
        .read(authStateProvider.notifier)
        .login(
          _serverController.text.trim(),
          _emailController.text.trim(),
          _passwordController.text,
        );
  }

  Future<void> _discoverServer() async {
    setState(() => _discovering = true);
    final url = await ServerDiscovery().discover();
    if (!mounted) return;
    if (url != null && _serverController.text.trim().isEmpty) {
      _serverController.text = url;
    }
    setState(() => _discovering = false);
  }

  Future<void> _startOAuth() async {
    final serverUrl = _serverController.text.trim();
    if (serverUrl.isEmpty) return;
    final url = await ref
        .read(authStateProvider.notifier)
        .startOAuth(serverUrl, _redirectUri);
    if (url == null) return;
    await launchUrl(Uri.parse(url), mode: LaunchMode.externalApplication);
  }

  void _listenForOAuthCallback() {
    final appLinks = AppLinks();
    appLinks.getInitialLink().then(_handleOAuthUri);
    _linkSub = appLinks.uriLinkStream.listen(_handleOAuthUri);
  }

  Future<void> _handleOAuthUri(Uri? uri) async {
    if (uri == null || uri.scheme != 'domus' || uri.host != 'oauth') return;
    final code = uri.queryParameters['code'];
    if (code == null || code.isEmpty) return;
    await ref
        .read(authStateProvider.notifier)
        .finishOAuth(
          serverUrl: _serverController.text.trim(),
          code: code,
          redirectUri: _redirectUri,
          stateValue: uri.queryParameters['state'],
        );
  }

  @override
  Widget build(BuildContext context) {
    final auth = ref.watch(authStateProvider);

    return Scaffold(
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 400),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  'Domus',
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.headlineLarge,
                ),
                const SizedBox(height: 32),
                TextField(
                  controller: _serverController,
                  decoration: const InputDecoration(
                    labelText: 'Server URL',
                    hintText: 'https://photos.example.com',
                  ),
                  keyboardType: TextInputType.url,
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _emailController,
                  decoration: const InputDecoration(labelText: 'Email'),
                  keyboardType: TextInputType.emailAddress,
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _passwordController,
                  decoration: const InputDecoration(labelText: 'Password'),
                  obscureText: true,
                  onSubmitted: (_) => _submit(),
                ),
                const SizedBox(height: 24),
                OutlinedButton.icon(
                  onPressed: _discovering ? null : _discoverServer,
                  icon: _discovering
                      ? const SizedBox.square(
                          dimension: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.radar_outlined),
                  label: const Text('Discover server'),
                ),
                const SizedBox(height: 12),
                FilledButton(
                  onPressed: auth.isLoading ? null : _submit,
                  child: auth.isLoading
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Login'),
                ),
                const SizedBox(height: 12),
                OutlinedButton.icon(
                  onPressed: auth.isLoading ? null : _startOAuth,
                  icon: const Icon(Icons.login_outlined),
                  label: const Text('Login with OAuth'),
                ),
                if (auth.error != null) ...[
                  const SizedBox(height: 12),
                  Text(
                    auth.error!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}
