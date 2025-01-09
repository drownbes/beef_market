{ self, pkgs }: let 
  testSelenium = pkgs.writers.writePython3Bin "test_geckodriver" {
    libraries = [ pkgs.python3Packages.selenium ]; 
  } ''
    from selenium import webdriver
    
    REMOTE_GECKODRIVER_URL = "http://127.0.0.1:4444"
    
    options = webdriver.FirefoxOptions()
    options.add_argument("-headless")
    options.binary_location = "/run/current-system/sw/bin/firefox"
    driver = webdriver.Remote(
        command_executor=REMOTE_GECKODRIVER_URL,
        options=options
    )
    driver.get("http://www.google.com")

    driver.quit()
  '';
in pkgs.nixosTest {
  name = "geckodriver_test";
  nodes.machine = { config, pkgs, ... }: {
    imports = [
      self.nixosModules.x86_64-linux.geckodriver
    ];

    services.geckodriver = {
      enable = true;
    };
    
    environment.systemPackages = [
      testSelenium
      pkgs.firefox
    ];

    system.stateVersion = "24.11";
  };

  extraPythonPackages = p: [ p.selenium ];

  skipTypeCheck = true;
  skipLint = true;

  testScript = ''
    machine.wait_for_unit("multi-user.target")
    machine.wait_for_unit("geckodriver.service")
    machine.wait_for_open_port(4444)
    machine.execute("test_geckodriver", check_return=True)
  '';
}
