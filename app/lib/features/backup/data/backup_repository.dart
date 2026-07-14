import 'package:dio/dio.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:photo_manager/photo_manager.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../../../core/api/api_client.dart';
import 'backup_settings.dart';

class BackupAlbum {
  const BackupAlbum({
    required this.id,
    required this.name,
    required this.count,
  });

  final String id;
  final String name;
  final int count;
}

class BackupRepository {
  BackupRepository(this._api);

  final ApiClient _api;

  Future<List<BackupAlbum>> albums() async {
    final permission = await PhotoManager.requestPermissionExtend();
    if (!permission.hasAccess) return const [];
    final paths = await PhotoManager.getAssetPathList(
      type: RequestType.common,
      filterOption: FilterOptionGroup(containsPathModified: true),
    );
    return [
      for (final path in paths)
        BackupAlbum(
          id: path.id,
          name: path.name,
          count: await path.assetCountAsync,
        ),
    ];
  }

  Future<int> runAutoBackup() async {
    return runAutoBackupWithClient(_api.dio);
  }

  Future<List<String>> uploadPickedFiles(
    List<PlatformFile> files, {
    void Function(int sent, int total)? onProgress,
  }) async {
    final uploadedIds = <String>[];
    for (final file in files) {
      final multipartFile = file.bytes != null
          ? MultipartFile.fromBytes(file.bytes!, filename: file.name)
          : await MultipartFile.fromFile(file.path!, filename: file.name);
      final now = DateTime.now().toUtc().toIso8601String();
      final form = FormData.fromMap({
        'assetData': multipartFile,
        'deviceAssetId': 'manual-${file.name}-${file.size}',
        'deviceId': 'domus-manual-upload',
        'fileCreatedAt': now,
        'fileModifiedAt': now,
        'isFavorite': 'false',
      });
      final response = await _api.dio.post<Map<String, dynamic>>(
        '/assets',
        data: form,
        onSendProgress: onProgress,
      );
      final id = response.data?['id'] as String?;
      if (id != null) {
        uploadedIds.add(id);
      }
    }
    return uploadedIds;
  }
}

Future<int> runAutoBackupWithClient(Dio dio) async {
  final prefs = await SharedPreferences.getInstance();
  if (!(prefs.getBool(BackupSettingsNotifier.autoBackupKey) ?? false)) {
    return 0;
  }
  final selectedAlbumIds =
      prefs.getStringList(BackupSettingsNotifier.selectedAlbumIdsKey) ??
      const [];
  if (selectedAlbumIds.isEmpty) return 0;

  final permission = await PhotoManager.requestPermissionExtend();
  if (!permission.hasAccess) return 0;

  final uploaded =
      (prefs.getStringList(BackupSettingsNotifier.uploadedDeviceAssetIdsKey) ??
              const [])
          .toSet();
  final paths = await PhotoManager.getAssetPathList(
    type: RequestType.common,
    filterOption: FilterOptionGroup(containsPathModified: true),
  );
  var uploadedCount = 0;
  for (final path in paths.where(
    (path) => selectedAlbumIds.contains(path.id),
  )) {
    final count = await path.assetCountAsync;
    for (var page = 0; page * 80 < count; page++) {
      final assets = await path.getAssetListPaged(page: page, size: 80);
      for (final asset in assets) {
        if (uploaded.contains(asset.id)) continue;
        final file = await asset.originFile;
        if (file == null) continue;
        final title = asset.title ?? await asset.titleAsync;
        final createdSecond =
            asset.createDateSecond ??
            DateTime.now().millisecondsSinceEpoch ~/ 1000;
        final modifiedSecond = asset.modifiedDateSecond ?? createdSecond;
        final createdAt = DateTime.fromMillisecondsSinceEpoch(
          createdSecond * 1000,
          isUtc: false,
        ).toUtc().toIso8601String();
        final modifiedAt = DateTime.fromMillisecondsSinceEpoch(
          modifiedSecond * 1000,
          isUtc: false,
        ).toUtc().toIso8601String();
        final form = FormData.fromMap({
          'assetData': await MultipartFile.fromFile(file.path, filename: title),
          'deviceAssetId': asset.id,
          'deviceId': 'domus-auto-backup',
          'fileCreatedAt': createdAt,
          'fileModifiedAt': modifiedAt,
          'isFavorite': '${asset.isFavorite}',
        });
        await dio.post<Map<String, dynamic>>('/assets', data: form);
        uploaded.add(asset.id);
        uploadedCount++;
        await prefs.setStringList(
          BackupSettingsNotifier.uploadedDeviceAssetIdsKey,
          uploaded.toList(),
        );
      }
    }
  }
  return uploadedCount;
}

final backupRepositoryProvider = Provider(
  (ref) => BackupRepository(ref.watch(apiClientProvider)),
);
