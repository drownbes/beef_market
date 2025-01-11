{ self, pkgs }: 
pkgs.nixosTest {
  name = "beef_market_test";
  nodes.machine = { config, pkgs, ... }: {
    imports = [
      self.nixosModules.beef_market
    ];

    services.beef_market = {
      enable = true;
    };
    environment.systemPackages = [
      pkgs.firefox
    ];

    system.stateVersion = "24.11";
  };

  skipTypeCheck = true;
  skipLint = true;

  testScript = ''
    machine.wait_for_unit("multi-user.target")
    machine.wait_for_unit("geckodriver.service")
    machine.wait_for_unit("beef_market.service")
  '';
}
