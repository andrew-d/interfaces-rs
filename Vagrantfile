# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
  # NOTE: Starting this VM might hang - you have to open the VM console and
  # Ctrl-C the "Starting vmware_guestd" line.
  # For now, this is commented out.
  ##config.vm.define "freebsd" do |bsd|
  ##  bsd.vm.guest = :freebsd
  ##  bsd.vm.synced_folder ".", "/vagrant", id: "vagrant-root", disabled: true
  ##  bsd.vm.box = "freebsd/FreeBSD-10.2-RELEASE"
  ##  bsd.ssh.shell = "sh"
  ##
  ##  bsd.vm.provider :vmware_fusion do |v|
  ##    v.vmx["memsize"] = "1024"
  ##    v.vmx["numvcpus"] = "1"
  ##  end
  ##end

  config.vm.define "linux" do |linux|
    linux.vm.box = "precise64_vmware"
    linux.vm.provision :shell, inline: <<-SHELL
      sudo apt-get update
      sudo apt-get install -yy curl git
      curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes
      multirust default stable
    SHELL
  end
end
