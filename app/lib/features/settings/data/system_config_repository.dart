import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/api/api_client.dart';

class StorageTemplateConfig {
  const StorageTemplateConfig({required this.enabled, required this.template});

  final bool enabled;
  final String template;

  factory StorageTemplateConfig.fromJson(Map<String, dynamic> json) {
    final storage =
        json['storageTemplate'] as Map<String, dynamic>? ?? const {};
    return StorageTemplateConfig(
      enabled: (storage['enabled'] as bool?) ?? false,
      template: (storage['template'] as String?) ?? '{{y}}/{{MM}}/{{filename}}',
    );
  }
}

class OAuthConfig {
  const OAuthConfig({
    required this.enabled,
    required this.authorizeUrl,
    required this.tokenEndpoint,
    required this.userinfoEndpoint,
    required this.clientId,
    required this.clientSecret,
    required this.scope,
  });

  final bool enabled;
  final String authorizeUrl;
  final String tokenEndpoint;
  final String userinfoEndpoint;
  final String clientId;
  final String clientSecret;
  final String scope;

  factory OAuthConfig.fromJson(Map<String, dynamic> json) {
    final oauth = json['oauth'] as Map<String, dynamic>? ?? const {};
    return OAuthConfig(
      enabled: (oauth['enabled'] as bool?) ?? false,
      authorizeUrl: (oauth['authorizeUrl'] as String?) ?? '',
      tokenEndpoint: (oauth['tokenEndpoint'] as String?) ?? '',
      userinfoEndpoint: (oauth['userinfoEndpoint'] as String?) ?? '',
      clientId: (oauth['clientId'] as String?) ?? '',
      clientSecret: (oauth['clientSecret'] as String?) ?? '',
      scope: (oauth['scope'] as String?) ?? 'openid email profile',
    );
  }
}

class SystemConfigRepository {
  SystemConfigRepository(this._api);

  final ApiClient _api;

  Future<StorageTemplateConfig> getStorageTemplate() async {
    final response = await _api.dio.get<Map<String, dynamic>>('/system-config');
    return StorageTemplateConfig.fromJson(response.data!);
  }

  Future<OAuthConfig> getOAuth() async {
    final response = await _api.dio.get<Map<String, dynamic>>('/system-config');
    return OAuthConfig.fromJson(response.data!);
  }

  Future<void> setStorageTemplate({
    required bool enabled,
    required String template,
  }) async {
    await _api.dio.put<void>(
      '/system-config',
      data: {
        'storageTemplate': {'enabled': enabled, 'template': template},
      },
    );
  }

  Future<void> setOAuth(OAuthConfig config) async {
    final existing = (await _api.dio.get<Map<String, dynamic>>(
      '/system-config',
    )).data!;
    existing['oauth'] = {
      'enabled': config.enabled,
      'authorizeUrl': config.authorizeUrl,
      'tokenEndpoint': config.tokenEndpoint,
      'userinfoEndpoint': config.userinfoEndpoint,
      'clientId': config.clientId,
      'clientSecret': config.clientSecret,
      'scope': config.scope,
    };
    await _api.dio.put<void>('/system-config', data: existing);
  }
}

final systemConfigRepositoryProvider = Provider(
  (ref) => SystemConfigRepository(ref.watch(apiClientProvider)),
);

final storageTemplateProvider = FutureProvider<StorageTemplateConfig>(
  (ref) => ref.watch(systemConfigRepositoryProvider).getStorageTemplate(),
);

final oauthConfigProvider = FutureProvider<OAuthConfig>(
  (ref) => ref.watch(systemConfigRepositoryProvider).getOAuth(),
);
