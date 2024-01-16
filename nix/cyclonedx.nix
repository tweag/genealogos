{ fetchFromGitHub
, buildDotnetModule

, lib
}:

buildDotnetModule rec {
  pname = "cyclonedx-cli";
  version = "0.25.0";

  src = fetchFromGitHub {
    owner = "CycloneDX";
    repo = pname;
    rev = "v${version}";
    hash = "sha256-kAMSdUMr/NhsbMBViFJQlzgUNnxWgi/CLb3CW9OpWFo=";
  };

  projectFile = "cyclonedx-cli.sln";

  nugetDeps = ./deps.nix;

  meta = with lib;
    {
      homepage = "https://github.com/CycloneDX/cyclonedx-cli";
      description = "tool for SBOM analysis, merging, diffs and format conversions";
      mainProgram = "cyclonedx";
      license = licenses.asl20;
    };
}
