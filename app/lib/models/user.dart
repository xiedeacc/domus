/// Mirrors Immich's UserResponseDto / LoginResponseDto fields we consume.
class User {
  const User({
    required this.id,
    required this.email,
    required this.name,
    required this.isAdmin,
    this.profileImagePath = '',
  });

  final String id;
  final String email;
  final String name;
  final bool isAdmin;
  final String profileImagePath;

  factory User.fromJson(Map<String, dynamic> json) => User(
    id: (json['userId'] ?? json['id']) as String,
    email: (json['userEmail'] ?? json['email']) as String,
    name: json['name'] as String,
    isAdmin: (json['isAdmin'] as bool?) ?? false,
    profileImagePath: (json['profileImagePath'] as String?) ?? '',
  );
}
