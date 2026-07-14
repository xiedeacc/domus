import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:workmanager/workmanager.dart';

const domusBackupTaskName = 'domus.backgroundBackup';
const domusBackupUniqueName = 'domus.periodicBackup';

class BackupSettings {
  const BackupSettings({
    required this.autoBackup,
    required this.backgroundBackup,
    required this.selectedAlbumIds,
    required this.uploadedDeviceAssetIds,
  });

  final bool autoBackup;
  final bool backgroundBackup;
  final List<String> selectedAlbumIds;
  final Set<String> uploadedDeviceAssetIds;

  BackupSettings copyWith({
    bool? autoBackup,
    bool? backgroundBackup,
    List<String>? selectedAlbumIds,
    Set<String>? uploadedDeviceAssetIds,
  }) => BackupSettings(
    autoBackup: autoBackup ?? this.autoBackup,
    backgroundBackup: backgroundBackup ?? this.backgroundBackup,
    selectedAlbumIds: selectedAlbumIds ?? this.selectedAlbumIds,
    uploadedDeviceAssetIds:
        uploadedDeviceAssetIds ?? this.uploadedDeviceAssetIds,
  );
}

class BackupSettingsNotifier extends AsyncNotifier<BackupSettings> {
  static const autoBackupKey = 'backup.autoBackup';
  static const backgroundBackupKey = 'backup.backgroundBackup';
  static const selectedAlbumIdsKey = 'backup.selectedAlbumIds';
  static const uploadedDeviceAssetIdsKey = 'backup.uploadedDeviceAssetIds';

  @override
  Future<BackupSettings> build() async => load();

  Future<BackupSettings> load() async {
    final prefs = await SharedPreferences.getInstance();
    return BackupSettings(
      autoBackup: prefs.getBool(autoBackupKey) ?? false,
      backgroundBackup: prefs.getBool(backgroundBackupKey) ?? false,
      selectedAlbumIds: prefs.getStringList(selectedAlbumIdsKey) ?? const [],
      uploadedDeviceAssetIds:
          (prefs.getStringList(uploadedDeviceAssetIdsKey) ?? const []).toSet(),
    );
  }

  Future<void> setAutoBackup(bool value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(autoBackupKey, value);
    state = AsyncData(
      (state.value ?? await load()).copyWith(autoBackup: value),
    );
  }

  Future<void> setBackgroundBackup(bool value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(backgroundBackupKey, value);
    if (value) {
      await Workmanager().registerPeriodicTask(
        domusBackupUniqueName,
        domusBackupTaskName,
        frequency: const Duration(minutes: 15),
        constraints: Constraints(networkType: NetworkType.connected),
        existingWorkPolicy: ExistingPeriodicWorkPolicy.update,
      );
    } else {
      await Workmanager().cancelByUniqueName(domusBackupUniqueName);
    }
    state = AsyncData(
      (state.value ?? await load()).copyWith(backgroundBackup: value),
    );
  }

  Future<void> setSelectedAlbumIds(List<String> value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(selectedAlbumIdsKey, value);
    state = AsyncData(
      (state.value ?? await load()).copyWith(selectedAlbumIds: value),
    );
  }
}

final backupSettingsProvider =
    AsyncNotifierProvider<BackupSettingsNotifier, BackupSettings>(
      BackupSettingsNotifier.new,
    );
