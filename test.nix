{ self, pkgs }:
pkgs.nixosTest {
  name = "geckodriver_test";
  nodes.machine = { config, pkgs, ... }: {
    imports = [
      self.nixosModules.x86_64-linux.geckodriver
    ];

    services.geckodriver = {
      enable = true;
    };

    system.stateVersion = "24.11";
  };

  testScript = ''
    machine.shell_interact()
  '';
}
